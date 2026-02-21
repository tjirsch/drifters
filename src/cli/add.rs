use crate::config::{AppConfig, LocalConfig, SyncMode, SyncRules};
use crate::error::Result;
use crate::git::{commit_and_push, EphemeralRepoGuard};
use std::io::{self, Write};
use std::path::PathBuf;

pub fn add_app(app_name: String) -> Result<()> {
    log::info!("Adding app: {}", app_name);

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo
    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Load sync rules
    let mut rules = SyncRules::load(repo_path)?;

    // Check if app already exists
    if rules.apps.contains_key(&app_name) {
        println!("App '{}' is already configured", app_name);
        return Ok(());
    }

    println!("Adding app '{}'", app_name);
    println!("Enter config file paths to sync (one per line, empty line to finish):");
    println!("Example: ~/.config/zed/settings.json");

    let mut files = Vec::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed = input.trim();

        if trimmed.is_empty() {
            break;
        }

        let path = PathBuf::from(trimmed);
        files.push(path);
        println!("  Added: {}", trimmed);
    }

    if files.is_empty() {
        println!("No files specified, cancelling");
        return Ok(());
    }

    // Ask for sync mode
    println!("\nSync mode:");
    println!("  1. Full - sync entire files (default)");
    println!("  2. Markers - sync only content between markers");
    print!("Choice [1]: ");
    io::stdout().flush()?;

    let mut mode_input = String::new();
    io::stdin().read_line(&mut mode_input)?;

    let sync_mode = match mode_input.trim() {
        "2" => SyncMode::Markers,
        _ => SyncMode::Full,
    };

    println!("Using sync mode: {:?}", sync_mode);

    // Create app config
    let app_config = AppConfig {
        files,
        sync_mode,
        exceptions: Default::default(),
        selectors: Default::default(),
    };

    // Add to rules
    rules.add_app(app_name.clone(), app_config);

    // Save rules
    rules.save(repo_path)?;
    println!("\n✓ Added '{}' to sync rules", app_name);

    // Commit and push
    println!("\nCommitting changes...");
    commit_and_push(
        repo_path,
        &format!("Add {} app from {}", app_name, config.machine_id),
    )?;

    println!("✓ Changes committed and pushed");
    println!("\nYou can now use:");
    println!("  drifters push {} - to push configs for this app", app_name);
    println!("  drifters pull {} - to pull configs for this app", app_name);

    Ok(())
}
