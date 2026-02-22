use crate::config::{LocalConfig, MachineRegistry, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, confirm_operation, EphemeralRepoGuard};

pub fn remove_machine(machine_id: String) -> Result<()> {
    log::info!("Removing machine '{}'", machine_id);

    // ── Load config and set up ephemeral repo ─────────────────────────────────
    let config = LocalConfig::load()?;

    println!("Fetching latest registry...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // ── Load registry and rules ───────────────────────────────────────────────
    let mut registry = MachineRegistry::load(repo_path)?;
    let mut rules = SyncRules::load(repo_path)?;

    // ── Validate machine_id exists ────────────────────────────────────────────
    if !registry.machines.contains_key(&machine_id) {
        let known: Vec<_> = registry.machines.keys().cloned().collect();
        return Err(DriftersError::Config(format!(
            "Machine '{}' is not registered in this repo.\nRegistered machines: {}",
            machine_id,
            if known.is_empty() {
                "(none)".to_string()
            } else {
                known.join(", ")
            }
        )));
    }

    // ── Determine if we're removing the current machine ──────────────────────
    let is_self = machine_id == config.machine_id;

    // ── Confirm with user ─────────────────────────────────────────────────────
    if is_self {
        eprintln!(
            "\n⚠️  WARNING: You are removing THIS machine ('{}').",
            machine_id
        );
        eprintln!(
            "   Your local drifters config (~/.config/drifters/config.toml) will be deleted."
        );
        eprintln!(
            "   You will need to run 'drifters init <repo-url>' to use drifters again."
        );
    } else {
        println!("\nRemove machine '{}'?", machine_id);
    }
    println!("This will:");
    println!(
        "  • Delete apps/<app>/machines/{id}/ in the repo (all apps)",
        id = machine_id
    );
    println!("  • Remove the machine from the registry (.drifters/machines.toml)");
    println!("  • Remove machine overrides from sync-rules.toml");
    if is_self {
        println!("  • Delete ~/.config/drifters/config.toml (local de-initialization)");
    }

    let prompt = format!(
        "{}Remove machine '{}'?",
        if is_self { "⚠️  " } else { "" },
        machine_id
    );
    // Default NO for both cases — extra caution for self-removal
    if !confirm_operation(&prompt, false)? {
        println!("Cancelled.");
        return Ok(());
    }

    // ── Delete machine directories in each app ────────────────────────────────
    let apps_dir = repo_path.join("apps");
    let mut dirs_removed = 0usize;

    if apps_dir.exists() {
        for entry in std::fs::read_dir(&apps_dir)? {
            let app_dir = entry?.path();
            if !app_dir.is_dir() {
                continue;
            }
            let machine_dir = app_dir.join("machines").join(&machine_id);
            if machine_dir.exists() {
                std::fs::remove_dir_all(&machine_dir)?;
                dirs_removed += 1;
                log::debug!("Deleted {:?}", machine_dir);
            }
        }
    }

    // ── Remove machine-specific overrides from SyncRules ──────────────────────
    let mut overrides_removed = 0usize;
    for app_config in rules.apps.values_mut() {
        if app_config.machines.remove(&machine_id).is_some() {
            overrides_removed += 1;
        }
    }

    // ── Remove entry from MachineRegistry ────────────────────────────────────
    registry.machines.remove(&machine_id);

    // ── Persist changes ───────────────────────────────────────────────────────
    registry.save(repo_path)?;
    rules.save(repo_path)?;

    // ── Commit and push ───────────────────────────────────────────────────────
    commit_and_push(
        repo_path,
        &format!("remove machine '{}'", machine_id),
    )?;

    // ── If removing self, delete local config ─────────────────────────────────
    if is_self {
        let config_path = LocalConfig::config_file_path()?;
        if config_path.exists() {
            std::fs::remove_file(&config_path)?;
        }
        println!(
            "\n✓ Machine '{}' removed and local drifters config deleted.",
            machine_id
        );
        println!("  Run 'drifters init <repo-url>' to re-initialize on this machine.");
    } else {
        println!("\n✓ Machine '{}' removed.", machine_id);
        if dirs_removed > 0 {
            println!("  Deleted config directories from {} app(s)", dirs_removed);
        }
        if overrides_removed > 0 {
            println!(
                "  Removed machine overrides from {} app(s)",
                overrides_removed
            );
        }
    }

    Ok(())
}
