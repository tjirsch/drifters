use crate::config::{LocalConfig, MachineRegistry, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, confirm_operation, EphemeralRepoGuard};

/// Remove an app's configs.
///
/// Behaviour depends on the flags:
///
/// * No flags  — removes this machine's uploaded configs for the app from the
///               repo. The app stays in sync-rules for all other machines.
/// * `--machine <id>` — same as above but for the named machine.
/// * `--all`   — removes the app from every machine: deletes `apps/<app>/`
///               entirely and removes the app from sync-rules.toml.
///               Requires confirmation; default NO.
pub fn remove_app(app_name: String, machine: Option<String>, all: bool) -> Result<()> {
    // --machine and --all are mutually exclusive
    if machine.is_some() && all {
        return Err(DriftersError::Config(
            "Cannot use --machine and --all together. \
             Use --machine <id> to remove one machine or --all to remove every machine."
                .to_string(),
        ));
    }

    log::info!("Removing app '{}' (machine={:?}, all={})", app_name, machine, all);

    let config = LocalConfig::load()?;

    println!("Fetching latest repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Guard: detect stale machine IDs
    crate::cli::common::verify_machine_registration(&config, repo_path)?;

    let mut rules = SyncRules::load(repo_path)?;

    if !rules.apps.contains_key(&app_name) {
        return Err(DriftersError::AppNotFound(app_name));
    }

    if all {
        remove_from_all(&app_name, &mut rules, repo_path)
    } else {
        let target = match machine {
            Some(ref id) => {
                // Validate the named machine exists in the registry
                let registry = MachineRegistry::load(repo_path)?;
                if !registry.machines.contains_key(id) {
                    let known: Vec<_> = registry.machines.keys().cloned().collect();
                    return Err(DriftersError::Config(format!(
                        "Machine '{}' is not registered in this repo.\n\
                         Registered machines: {}",
                        id,
                        if known.is_empty() {
                            "(none)".to_string()
                        } else {
                            known.join(", ")
                        }
                    )));
                }
                id.clone()
            }
            None => config.machine_id.clone(),
        };
        remove_from_machine(&app_name, &target, &config.machine_id, &mut rules, repo_path)
    }
}

/// Remove a single machine's uploaded configs for `app_name`.
fn remove_from_machine(
    app_name: &str,
    target_machine: &str,
    local_machine: &str,
    rules: &mut SyncRules,
    repo_path: &std::path::Path,
) -> Result<()> {
    let repo_path_buf = repo_path.to_path_buf();
    let machine_dir = repo_path
        .join("apps")
        .join(app_name)
        .join("machines")
        .join(target_machine);

    if machine_dir.exists() {
        std::fs::remove_dir_all(&machine_dir)?;
        println!(
            "  Deleted uploaded configs for '{}' on machine '{}'",
            app_name, target_machine
        );
    } else {
        println!(
            "  No uploaded configs found for '{}' on machine '{}' (nothing to delete)",
            app_name, target_machine
        );
    }

    // Remove any machine-specific overrides from sync-rules for this machine
    if let Some(app_config) = rules.apps.get_mut(app_name) {
        app_config.machines.remove(target_machine);
    }
    rules.save(&repo_path_buf)?;

    let is_self = target_machine == local_machine;
    let commit_msg = if is_self {
        format!("remove {} configs from machine {}", app_name, target_machine)
    } else {
        format!(
            "remove {} configs from machine {} (via {})",
            app_name, target_machine, local_machine
        )
    };
    commit_and_push(&repo_path_buf, &commit_msg)?;

    println!(
        "\n✓ Removed '{}' configs from machine '{}'.",
        app_name, target_machine
    );
    println!(
        "  The app remains configured in sync-rules for all other machines."
    );
    println!(
        "  Run 'drifters remove-app {} --all' to remove it from every machine.",
        app_name
    );
    Ok(())
}

/// Remove `app_name` from every machine and from sync-rules entirely.
fn remove_from_all(
    app_name: &str,
    rules: &mut SyncRules,
    repo_path: &std::path::Path,
) -> Result<()> {
    let repo_path_buf = repo_path.to_path_buf();
    eprintln!(
        "\n⚠️  This will remove '{}' from ALL machines.",
        app_name
    );
    eprintln!("   • Deletes apps/{}/  (all uploaded configs in the repo)", app_name);
    eprintln!("   • Removes the app from sync-rules.toml");
    eprintln!("   Note: local config files on each machine are NOT deleted.");

    if !confirm_operation(&format!("Remove '{}' from all machines?", app_name), false)? {
        println!("Cancelled.");
        return Ok(());
    }

    // Delete apps/<app>/ entirely
    let app_dir = repo_path.join("apps").join(app_name);
    if app_dir.exists() {
        std::fs::remove_dir_all(&app_dir)?;
        log::debug!("Deleted {:?}", app_dir);
    }

    // Remove from sync-rules
    rules.apps.remove(app_name);
    rules.save(&repo_path_buf)?;

    commit_and_push(
        &repo_path_buf,
        &format!("remove {} app from all machines", app_name),
    )?;

    println!("\n✓ Removed '{}' from all machines and sync-rules.", app_name);
    println!("  Local config files on each machine have NOT been deleted.");
    Ok(())
}
