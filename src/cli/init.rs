use crate::config::{LocalConfig, MachineRegistry, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{clone_repo, commit_and_push, init_repo};
use std::io::{self, Write};
use std::path::PathBuf;

pub fn initialize(repo_url: String) -> Result<()> {
    log::info!("Initializing drifters with repo: {}", repo_url);

    // Check if already initialized
    if let Ok(_) = LocalConfig::load() {
        return Err(DriftersError::Config(
            "Drifters already initialized. Check ~/.config/drifters/config.toml".to_string(),
        ));
    }

    // Detect machine ID
    let detected_id = LocalConfig::detect_machine_id();
    println!("Detected machine: {} ({})", detected_id, std::env::consts::OS);

    // Ask for confirmation or override
    print!("Use this machine ID? [Y/n]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let machine_id = if input.trim().to_lowercase() == "n" || input.trim().to_lowercase() == "no" {
        print!("Enter machine ID: ");
        io::stdout().flush()?;
        let mut custom_id = String::new();
        io::stdin().read_line(&mut custom_id)?;
        custom_id.trim().to_string()
    } else {
        detected_id
    };

    println!("Using machine ID: {}", machine_id);

    // Determine repo path
    let repo_path = get_repo_path()?;

    println!("Repository will be cloned to: {:?}", repo_path);

    // Clone or init repository
    let is_new_repo = if repo_path.exists() {
        println!("Repository directory already exists");
        false
    } else {
        println!("Cloning repository...");

        match clone_repo(&repo_url, &repo_path) {
            Ok(_) => {
                println!("✓ Repository cloned successfully");
                false
            }
            Err(e) => {
                // If clone fails, try initializing an empty repo
                log::warn!("Clone failed ({}), initializing empty repository", e);
                println!("Clone failed, initializing empty repository...");
                let repo = init_repo(&repo_path)?;

                // Set up remote origin
                repo.remote("origin", &repo_url)?;
                println!("✓ Empty repository initialized with remote");
                true
            }
        }
    };

    // Create local config (without repo_path, it's ephemeral)
    let local_config = LocalConfig::new(machine_id.clone(), repo_url.clone());
    local_config.save()?;
    println!("✓ Local config saved to {:?}", LocalConfig::config_file_path()?);

    // Load or create machine registry
    let mut registry = if is_new_repo {
        MachineRegistry::new()
    } else {
        MachineRegistry::load(&repo_path).unwrap_or_else(|_| MachineRegistry::new())
    };

    // Register this machine
    let os = MachineRegistry::detect_os();
    registry.register_machine(machine_id.clone(), os.clone());
    registry.save(&repo_path)?;
    println!("✓ Registered machine '{}' ({})", machine_id, os);

    // Create sync rules if new repo
    if is_new_repo {
        let rules = SyncRules::new();
        rules.save(&repo_path)?;
        println!("✓ Created sync-rules.toml");
    }

    // Commit and push if we made changes
    if is_new_repo || registry.machines.len() == 1 {
        println!("\nCommitting changes...");
        commit_and_push(&repo_path, &format!("Initialize drifters on {}", machine_id))?;
        println!("✓ Changes committed and pushed");
    }

    // Clean up ephemeral repo
    std::fs::remove_dir_all(&repo_path)?;
    log::debug!("Cleaned up temporary repo at {:?}", repo_path);

    // Ask about shell hook
    println!("\nSetup complete!");
    println!("\nTo enable auto-sync on shell startup, add this to your .zshrc or .bashrc:");
    println!("  eval \"$(drifters hook)\"");

    print!("\nAdd shell hook now? [y/N]: ");
    io::stdout().flush()?;
    let mut hook_input = String::new();
    io::stdin().read_line(&mut hook_input)?;

    if hook_input.trim().to_lowercase() == "y" || hook_input.trim().to_lowercase() == "yes" {
        add_shell_hook()?;
    }

    Ok(())
}

fn get_repo_path() -> Result<PathBuf> {
    // For init, we use a temporary location that will be cleaned up
    LocalConfig::get_temp_repo_path()
}

fn add_shell_hook() -> Result<()> {
    // Detect shell
    let shell = std::env::var("SHELL").unwrap_or_default();

    let rc_file = if shell.contains("zsh") {
        dirs::home_dir().map(|h| h.join(".zshrc"))
    } else if shell.contains("bash") {
        dirs::home_dir().map(|h| h.join(".bashrc"))
    } else {
        None
    };

    if let Some(rc_path) = rc_file {
        let hook_line = "\n# Drifters auto-sync\neval \"$(drifters hook)\"\n";

        // Check if already added
        if rc_path.exists() {
            let contents = std::fs::read_to_string(&rc_path)?;
            if contents.contains("drifters hook") {
                println!("Shell hook already present in {:?}", rc_path);
                return Ok(());
            }

            // Back up the existing file before modifying it
            let backup_path = rc_path.with_extension("bak");
            std::fs::copy(&rc_path, &backup_path)?;
            log::debug!("Backed up {:?} to {:?}", rc_path, backup_path);
        }

        // Append hook
        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc_path)?;
        file.write_all(hook_line.as_bytes())?;

        println!("✓ Added shell hook to {:?}", rc_path);
        println!("  Run 'source {:?}' or restart your shell to activate", rc_path);
    } else {
        println!("Could not detect shell config file");
        println!("Manually add this to your shell config:");
        println!("  eval \"$(drifters hook)\"");
    }

    Ok(())
}
