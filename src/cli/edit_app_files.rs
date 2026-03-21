use crate::cli::common::open_file;
use crate::config::fileset::resolve_fileset;
use crate::config::{LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::EphemeralRepoGuard;
use std::io::{self, Write};

pub fn edit_app_files(app_name: &str) -> Result<()> {
    let config = LocalConfig::load()?;

    // Load rules from repo
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let rules = SyncRules::load(repo_guard.path())?;

    let app_config = rules
        .apps
        .get(app_name)
        .ok_or_else(|| DriftersError::AppNotFound(app_name.to_string()))?;

    // Resolve files present on this machine
    let fileset = resolve_fileset(app_config, &config.machine_id, std::env::consts::OS)?;

    let existing: Vec<_> = fileset.into_iter().filter(|p| p.exists()).collect();

    if existing.is_empty() {
        println!("No files found for '{}' on this machine.", app_name);
        return Ok(());
    }

    println!("Files for '{}':\n", app_name);
    for (i, path) in existing.iter().enumerate() {
        println!("  [{}] {}", i + 1, path.display());
    }
    println!("  [0] Cancel\n");

    print!("Choose a file to open: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let choice: usize = match input.trim().parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid choice.");
            return Ok(());
        }
    };

    if choice == 0 {
        println!("Cancelled.");
        return Ok(());
    }

    if choice > existing.len() {
        println!("Invalid choice.");
        return Ok(());
    }

    let selected = &existing[choice - 1];
    open_file(selected, config.editor.as_deref())?;

    Ok(())
}
