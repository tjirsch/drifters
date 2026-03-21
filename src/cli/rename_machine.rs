use crate::config::{LocalConfig, MachineRegistry, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, confirm_operation, EphemeralRepoGuard};
use std::io::{self, Write};

pub fn rename_machine(old_id: String, new_id: String) -> Result<()> {
    log::info!("Renaming machine '{}' → '{}'", old_id, new_id);

    // Validate new_id
    if new_id.is_empty() {
        return Err(DriftersError::Config(
            "New machine ID cannot be empty.".to_string(),
        ));
    }
    if new_id.contains('/') || new_id.contains('\\') {
        return Err(DriftersError::Config(
            "New machine ID cannot contain '/' or '\\'.".to_string(),
        ));
    }
    if new_id == old_id {
        return Err(DriftersError::Config(format!(
            "New machine ID is the same as the current one ('{}').",
            old_id
        )));
    }

    let mut config = LocalConfig::load()?;

    println!("Fetching latest registry...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    let mut registry = MachineRegistry::load(repo_path)?;
    let mut rules = SyncRules::load(repo_path)?;

    if !registry.machines.contains_key(&old_id) {
        let known: Vec<_> = registry.machines.keys().cloned().collect();
        return Err(DriftersError::Config(format!(
            "Machine '{}' is not registered in this repo.\nRegistered machines: {}",
            old_id,
            if known.is_empty() {
                "(none)".to_string()
            } else {
                known.join(", ")
            }
        )));
    }

    if registry.machines.contains_key(&new_id) {
        return Err(DriftersError::Config(format!(
            "Machine ID '{}' is already registered. Choose a different ID.",
            new_id
        )));
    }

    let old_branch = format!("machines/{}", old_id);
    let new_branch = format!("machines/{}", new_id);

    println!(
        "\nRename machine '{}' → '{}'",
        old_id, new_id
    );
    println!("This will:");
    println!("  • Rename branch '{}' → '{}'", old_branch, new_branch);
    println!("  • Update the machine registry (.drifters/machines.toml)");
    println!("  • Update machine overrides in sync-rules.toml");
    if old_id == config.machine_id {
        println!(
            "  • Update your local config (~/.config/drifters/drifters.toml) to '{}'",
            new_id
        );
    }
    io::stdout().flush()?;

    if !confirm_operation("Proceed with rename?", false)? {
        println!("Cancelled.");
        return Ok(());
    }

    // Rename the branch: create new from old, delete old
    // First, fetch the old branch
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["fetch", "origin", &old_branch])
        .output();

    // Create new branch from old
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["branch", &new_branch, &format!("origin/{}", old_branch)])
        .output();

    // Push new branch
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["push", "-u", "origin", &new_branch])
        .output();

    // Delete old remote branch
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["push", "origin", "--delete", &old_branch])
        .output();

    // Delete old local branch
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["branch", "-D", &old_branch])
        .output();

    println!("  ✓ Branch renamed: {} → {}", old_branch, new_branch);

    // Rename machine-specific overrides in SyncRules
    let mut overrides_renamed = 0usize;
    for app_config in rules.apps.values_mut() {
        if let Some(info) = app_config.machines.remove(&old_id) {
            app_config.machines.insert(new_id.clone(), info);
            overrides_renamed += 1;
        }
    }

    // Rename entry in MachineRegistry
    let mut machine_info = registry.machines.remove(&old_id).ok_or_else(|| {
        DriftersError::Config(format!(
            "Machine '{}' disappeared from registry during rename",
            old_id
        ))
    })?;
    machine_info.branch = Some(new_branch.clone());
    registry.machines.insert(new_id.clone(), machine_info);

    // Persist changes
    registry.save(repo_path)?;
    rules.save(repo_path)?;

    // Commit and push
    commit_and_push(
        repo_path,
        &format!("rename machine '{}' to '{}'", old_id, new_id),
    )?;

    // Update local config if this is the current machine
    if old_id == config.machine_id {
        config.machine_id = new_id.clone();
        config.save()?;
        println!(
            "✓ Updated local config machine ID to '{}'",
            new_id
        );
    }

    println!(
        "\n✓ Machine '{}' renamed to '{}'",
        old_id, new_id
    );
    if overrides_renamed > 0 {
        println!(
            "  Updated machine overrides in {} app(s)",
            overrides_renamed
        );
    }

    Ok(())
}
