use crate::config::{LocalConfig, MachineRegistry, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, confirm_operation, EphemeralRepoGuard};
use std::io::{self, Write};

pub fn rename_machine(old_id: String, new_id: String) -> Result<()> {
    log::info!("Renaming machine '{}' → '{}'", old_id, new_id);

    // ── Validate new_id before touching anything ──────────────────────────────
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

    // ── Load config and set up ephemeral repo ─────────────────────────────────
    let mut config = LocalConfig::load()?;

    println!("Fetching latest registry...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // ── Load registry and rules ───────────────────────────────────────────────
    let mut registry = MachineRegistry::load(repo_path)?;
    let mut rules = SyncRules::load(repo_path)?;

    // ── Validate old_id exists ────────────────────────────────────────────────
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

    // ── Validate new_id is not already taken ─────────────────────────────────
    if registry.machines.contains_key(&new_id) {
        return Err(DriftersError::Config(format!(
            "Machine ID '{}' is already registered. Choose a different ID.",
            new_id
        )));
    }

    // ── Confirm with user ─────────────────────────────────────────────────────
    println!(
        "\nRename machine '{}' → '{}'",
        old_id, new_id
    );
    println!("This will:");
    println!(
        "  • Rename apps/<app>/machines/{old}/ → apps/<app>/machines/{new}/ in the repo",
        old = old_id,
        new = new_id
    );
    println!("  • Update the machine registry (.drifters/machines.toml)");
    println!("  • Update machine overrides in sync-rules.toml");
    if old_id == config.machine_id {
        println!(
            "  • Update your local config (~/.config/drifters/config.toml) to '{}'",
            new_id
        );
    }
    io::stdout().flush()?;

    let msg = format!("Proceed with rename?");
    if !confirm_operation(&msg, false)? {
        println!("Cancelled.");
        return Ok(());
    }

    // ── Rename directories in each app ────────────────────────────────────────
    let apps_dir = repo_path.join("apps");
    let mut dirs_renamed = 0usize;

    if apps_dir.exists() {
        for entry in std::fs::read_dir(&apps_dir)? {
            let app_dir = entry?.path();
            if !app_dir.is_dir() {
                continue;
            }
            let old_machine_dir = app_dir.join("machines").join(&old_id);
            let new_machine_dir = app_dir.join("machines").join(&new_id);
            if old_machine_dir.exists() {
                std::fs::rename(&old_machine_dir, &new_machine_dir)?;
                dirs_renamed += 1;
                log::debug!(
                    "Renamed {:?} → {:?}",
                    old_machine_dir,
                    new_machine_dir
                );
            }
        }
    }

    // ── Rename machine-specific overrides in SyncRules ────────────────────────
    let mut overrides_renamed = 0usize;
    for app_config in rules.apps.values_mut() {
        if let Some(info) = app_config.machines.remove(&old_id) {
            app_config.machines.insert(new_id.clone(), info);
            overrides_renamed += 1;
        }
    }

    // ── Rename entry in MachineRegistry ──────────────────────────────────────
    let machine_info = registry.machines.remove(&old_id).ok_or_else(|| {
        DriftersError::Config(format!(
            "Machine '{}' disappeared from registry during rename",
            old_id
        ))
    })?;
    registry.machines.insert(new_id.clone(), machine_info);

    // ── Persist changes ───────────────────────────────────────────────────────
    registry.save(repo_path)?;
    rules.save(repo_path)?;

    // ── Commit and push ───────────────────────────────────────────────────────
    commit_and_push(
        repo_path,
        &format!("rename machine '{}' to '{}'", old_id, new_id),
    )?;

    // ── Update local config if this is the current machine ────────────────────
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
    if dirs_renamed > 0 {
        println!(
            "  Renamed config directories in {} app(s)",
            dirs_renamed
        );
    }
    if overrides_renamed > 0 {
        println!(
            "  Updated machine overrides in {} app(s)",
            overrides_renamed
        );
    }

    Ok(())
}
