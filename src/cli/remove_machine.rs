use crate::config::{LocalConfig, MachineRegistry, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, confirm_operation, EphemeralRepoGuard};

pub fn remove_machine(machine_id: String) -> Result<()> {
    log::info!("Removing machine '{}'", machine_id);

    let config = LocalConfig::load()?;

    println!("Fetching latest registry...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    let mut registry = MachineRegistry::load(repo_path)?;
    let mut rules = SyncRules::load(repo_path)?;

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

    let is_self = machine_id == config.machine_id;
    let machine_branch = format!("machines/{}", machine_id);

    if is_self {
        eprintln!(
            "\n⚠️  WARNING: You are removing THIS machine ('{}').",
            machine_id
        );
        eprintln!(
            "   Your local drifters config (~/.config/drifters/drifters.toml) will be deleted."
        );
        eprintln!(
            "   You will need to run 'drifters init <repo-url>' to use drifters again."
        );
    } else {
        println!("\nRemove machine '{}'?", machine_id);
    }
    println!("This will:");
    println!("  • Delete branch '{}'", machine_branch);
    println!("  • Remove the machine from the registry (.drifters/machines.toml)");
    println!("  • Remove machine overrides from sync-rules.toml");
    if is_self {
        println!("  • Delete ~/.config/drifters/drifters.toml (local de-initialization)");
    }

    let prompt = format!(
        "{}Remove machine '{}'?",
        if is_self { "⚠️  " } else { "" },
        machine_id
    );
    if !confirm_operation(&prompt, false)? {
        println!("Cancelled.");
        return Ok(());
    }

    // Delete the machine branch (remote)
    let delete_result = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["push", "origin", "--delete", &machine_branch])
        .output();

    match delete_result {
        Ok(output) if output.status.success() => {
            println!("  ✓ Deleted remote branch '{}'", machine_branch);
        }
        _ => {
            log::warn!("Could not delete remote branch '{}'", machine_branch);
            println!("  ⚠️  Could not delete remote branch '{}' (may not exist)", machine_branch);
        }
    }

    // Delete local branch if it exists
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["branch", "-D", &machine_branch])
        .output();

    // Remove machine-specific overrides from SyncRules
    let mut overrides_removed = 0usize;
    for app_config in rules.apps.values_mut() {
        if app_config.machines.remove(&machine_id).is_some() {
            overrides_removed += 1;
        }
    }

    // Remove entry from MachineRegistry
    registry.machines.remove(&machine_id);

    // Persist changes
    registry.save(repo_path)?;
    rules.save(repo_path)?;

    // Commit and push
    commit_and_push(
        repo_path,
        &format!("remove machine '{}'", machine_id),
    )?;

    // If removing self, delete local config
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
        if overrides_removed > 0 {
            println!(
                "  Removed machine overrides from {} app(s)",
                overrides_removed
            );
        }
    }

    Ok(())
}
