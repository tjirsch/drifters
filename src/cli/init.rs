use crate::config::{LocalConfig, MachineRegistry, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{clone_repo, commit_and_push, create_branch, init_repo, set_remote_origin};
use std::io::{self, Write};
use std::path::PathBuf;

/// RAII guard that deletes a directory tree on Drop.
struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        if self.0.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.0) {
                log::warn!("Failed to clean up temp dir {:?}: {}", self.0, e);
            } else {
                log::debug!("Cleaned up temporary repo at {:?}", self.0);
            }
        }
    }
}

pub fn initialize(repo_url: String) -> Result<()> {
    log::info!("Initializing drifters with repo: {}", repo_url);

    // Check if already initialized
    if let Ok(_) = LocalConfig::load() {
        return Err(DriftersError::Config(
            "Drifters already initialized. Check ~/.config/drifters/drifters.toml".to_string(),
        ));
    }

    // Detect machine ID (hostname)
    let detected_id = LocalConfig::detect_machine_id();
    println!("Detected machine: {} ({})", detected_id, std::env::consts::OS);

    // Determine repo path
    let repo_path = get_repo_path()?;
    println!("Repository will be cloned to: {:?}", repo_path);

    // RAII cleanup guard
    let _cleanup = TempDirGuard(repo_path.clone());

    // Clone or init repository
    if !repo_path.exists() {
        println!("Cloning repository...");

        match clone_repo(&repo_url, &repo_path) {
            Ok(_) => {
                println!("✓ Repository cloned successfully");
            }
            Err(e) => {
                log::warn!("Clone failed ({}), initializing empty repository", e);
                println!("Clone failed, initializing empty repository...");
                init_repo(&repo_path)?;
                set_remote_origin(&repo_path, &repo_url)?;
                println!("✓ Empty repository initialized with remote");
            }
        }
    } else {
        println!("Repository directory already exists");
    }

    // Detect whether this repo needs bootstrapping (no .drifters/ dir means
    // either a fresh init or an empty clone with no commits yet)
    let needs_bootstrap = !repo_path.join(".drifters").exists();

    // Load or create machine registry
    let mut registry = if needs_bootstrap {
        MachineRegistry::new()
    } else {
        MachineRegistry::load(&repo_path).unwrap_or_else(|_| MachineRegistry::new())
    };

    // Resolve machine ID
    let machine_id = resolve_machine_id(&detected_id, &registry)?;
    println!("Using machine ID: {}", machine_id);

    let machine_branch = format!("machines/{}", machine_id);

    // Create local config
    let local_config = LocalConfig::new(machine_id.clone(), repo_url.clone());
    local_config.save()?;
    println!("✓ Local config saved to {:?}", LocalConfig::config_file_path()?);

    // Register this machine (with branch info)
    let os = MachineRegistry::detect_os();
    registry.register_machine(machine_id.clone(), os.clone());
    registry.save(&repo_path)?;
    println!("✓ Registered machine '{}' ({}) on branch '{}'", machine_id, os, machine_branch);

    // Create sync rules if repo needs bootstrapping
    if needs_bootstrap {
        let rules = SyncRules::new();
        rules.save(&repo_path)?;
        println!("✓ Created sync-rules.toml");
    }

    // Commit and push to main first (machine registration must be on main)
    println!("\nCommitting changes to main...");
    commit_and_push(&repo_path, &format!("Initialize drifters on {}", machine_id))?;
    println!("✓ Changes committed and pushed to main");

    // Create the machine branch from main
    println!("Creating machine branch '{}'...", machine_branch);
    create_branch(&repo_path, &machine_branch)?;

    // Push the machine branch to remote
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&repo_path)
        .args(["push", "-u", "origin", &machine_branch])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        log::warn!("Failed to push machine branch: {}", stderr);
        println!("⚠️  Could not push machine branch (will be pushed on first push-app)");
    } else {
        println!("✓ Machine branch '{}' created and pushed", machine_branch);
    }

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

fn resolve_machine_id(detected: &str, registry: &MachineRegistry) -> Result<String> {
    if !registry.machines.contains_key(detected) {
        print!("Use machine ID '{}' ? [Y/n]: ", detected);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();
        if answer != "n" && answer != "no" {
            return Ok(detected.to_string());
        }
        println!("Enter a custom machine ID:");
    } else {
        println!(
            "⚠️  Machine ID '{}' is already registered in this repo.",
            detected
        );
        println!("Please choose a unique ID for this machine.");
    }

    for attempt in 1..=3usize {
        print!("Enter a unique machine ID (attempt {}/3): ", attempt);
        io::stdout().flush()?;
        let mut custom = String::new();
        io::stdin().read_line(&mut custom)?;
        let custom = custom.trim().to_string();

        if custom.is_empty() {
            eprintln!("  Machine ID cannot be empty.");
        } else if custom.contains('/') || custom.contains('\\') {
            eprintln!("  Machine ID cannot contain '/' or '\\'.");
        } else if registry.machines.contains_key(&custom) {
            eprintln!("  '{}' is already taken.", custom);
        } else {
            return Ok(custom);
        }
    }

    Err(DriftersError::Config(
        "Could not choose a unique machine ID after 3 attempts. \
         Re-run `drifters init` and pick a different ID."
            .to_string(),
    ))
}

fn get_repo_path() -> Result<PathBuf> {
    LocalConfig::get_temp_repo_path()
}

fn add_shell_hook() -> Result<()> {
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

        if rc_path.exists() {
            let contents = std::fs::read_to_string(&rc_path)?;
            if contents.contains("drifters hook") {
                println!("Shell hook already present in {:?}", rc_path);
                return Ok(());
            }

            let backup_path = rc_path.with_file_name(format!(
                "{}.bak",
                rc_path
                    .file_name()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_default()
            ));
            std::fs::copy(&rc_path, &backup_path)?;
            log::debug!("Backed up {:?} to {:?}", rc_path, backup_path);
        }

        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc_path)?;
        file.write_all(hook_line.as_bytes())?;

        println!("✓ Added shell hook to {:?}", rc_path);
        println!(
            "  Run 'source {:?}' or restart your shell to activate",
            rc_path
        );
    } else {
        println!("Could not detect shell config file");
        println!("Manually add this to your shell config:");
        println!("  eval \"$(drifters hook)\"");
    }

    Ok(())
}
