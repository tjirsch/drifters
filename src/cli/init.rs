use crate::config::{LocalConfig, MachineRegistry, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{clone_repo, commit_and_push, init_repo};
use std::io::{self, Write};
use std::path::PathBuf;

/// RAII guard that deletes a directory tree on Drop.
///
/// Used in `initialize` to ensure the ephemeral temp repo is always cleaned
/// up, even when an early `return Err(...)` is hit after the clone/init.
/// `EphemeralRepoGuard` cannot be reused here because it also calls
/// `pull_latest`, which is wrong during a fresh init.
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
            "Drifters already initialized. Check ~/.config/drifters/config.toml".to_string(),
        ));
    }

    // Detect machine ID (hostname)
    let detected_id = LocalConfig::detect_machine_id();
    println!("Detected machine: {} ({})", detected_id, std::env::consts::OS);

    // Determine repo path
    let repo_path = get_repo_path()?;
    println!("Repository will be cloned to: {:?}", repo_path);

    // RAII cleanup guard — deletes repo_path on Drop, even on early return
    let _cleanup = TempDirGuard(repo_path.clone());

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

    // Load or create machine registry.
    // We load this BEFORE selecting the machine ID so we can check for
    // collisions — two machines with the same hostname would otherwise
    // silently overwrite each other's configs.
    let mut registry = if is_new_repo {
        MachineRegistry::new()
    } else {
        MachineRegistry::load(&repo_path).unwrap_or_else(|_| MachineRegistry::new())
    };

    // Resolve machine ID — uses hostname if unique, prompts if already taken
    let machine_id = resolve_machine_id(&detected_id, &registry)?;
    println!("Using machine ID: {}", machine_id);

    // Create local config (without repo_path, it's ephemeral)
    let local_config = LocalConfig::new(machine_id.clone(), repo_url.clone());
    local_config.save()?;
    println!("✓ Local config saved to {:?}", LocalConfig::config_file_path()?);

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

    // Always commit and push — machine registration must be recorded in the
    // repo even when joining an existing repo (not just for the first machine).
    // commit_and_push is a no-op if the tree hasn't changed (see Fix 12).
    println!("\nCommitting changes...");
    commit_and_push(&repo_path, &format!("Initialize drifters on {}", machine_id))?;
    println!("✓ Changes committed and pushed");
    // Temp repo is cleaned up by `_cleanup` (TempDirGuard) when it goes out
    // of scope at the end of this function.

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

/// Returns a machine ID that is unique within the given registry.
///
/// Uses the detected hostname if it is not already registered;
/// prompts the user to choose a different ID if it is taken.
///
/// # Future extensions
/// TODO(future): add `remove-machine` and `rename-machine` commands.
/// These are non-trivial because the machine ID is used as a directory
/// name inside `apps/<app>/machines/<id>/` in the repo, and a naive
/// rename/delete must be scoped to those paths to avoid colliding with
/// app names that happen to share the same string (e.g. a machine named
/// "zed" and an app named "zed").
fn resolve_machine_id(detected: &str, registry: &MachineRegistry) -> Result<String> {
    if !registry.machines.contains_key(detected) {
        // Happy path: hostname is free — confirm or let user override
        print!("Use machine ID '{}' ? [Y/n]: ", detected);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();
        if answer != "n" && answer != "no" {
            return Ok(detected.to_string());
        }
        // User wants a different name even though it's available
        println!("Enter a custom machine ID:");
    } else {
        println!(
            "⚠️  Machine ID '{}' is already registered in this repo.",
            detected
        );
        println!("Please choose a unique ID for this machine.");
    }

    // Prompt for a unique custom ID (up to 3 attempts)
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

            // Back up the existing file before modifying it. Use appended .bak
            // so .zshrc -> .zshrc.bak; with_extension("bak") would produce .bak and lose the name.
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

        // Append hook
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
