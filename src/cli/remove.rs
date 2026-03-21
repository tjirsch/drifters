use crate::config::{LocalConfig, MachineRegistry, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{
    checkout_branch, commit_and_push, confirm_operation, EphemeralRepoGuard,
};

/// Remove an app's configs.
///
/// * No flags  — removes this machine's uploaded configs for the app from its branch.
/// * `--machine <id>` — same but for the named machine's branch.
/// * `--all`   — removes the app from every branch and from sync-rules.toml on main.
pub fn remove_app(app_name: String, machine: Option<String>, all: bool) -> Result<()> {
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

    let rules = SyncRules::load(repo_path)?;

    if !rules.apps.contains_key(&app_name) {
        return Err(DriftersError::AppNotFound(app_name));
    }

    if all {
        remove_from_all(&app_name, repo_path, &config)
    } else {
        let target = match machine {
            Some(ref id) => {
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
        remove_from_machine(&app_name, &target, &config.machine_id, repo_path)
    }
}

/// Remove a single machine's uploaded configs for `app_name` from its branch.
fn remove_from_machine(
    app_name: &str,
    target_machine: &str,
    local_machine: &str,
    repo_path: &std::path::Path,
) -> Result<()> {
    let repo_path_buf = repo_path.to_path_buf();
    let machine_branch = format!("machines/{}", target_machine);

    // Switch to the machine's branch
    if crate::git::checkout_or_create_branch(&repo_path_buf, &machine_branch, "main").is_err() {
        println!(
            "  No branch found for machine '{}' (nothing to delete)",
            target_machine
        );
        return Ok(());
    }

    let app_dir = repo_path.join("apps").join(app_name);
    if app_dir.exists() {
        std::fs::remove_dir_all(&app_dir)?;
        println!(
            "  Deleted uploaded configs for '{}' on branch '{}'",
            app_name, machine_branch
        );
    } else {
        println!(
            "  No uploaded configs found for '{}' on branch '{}' (nothing to delete)",
            app_name, machine_branch
        );
    }

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
    Ok(())
}

/// Remove `app_name` from every machine and from sync-rules entirely.
fn remove_from_all(
    app_name: &str,
    repo_path: &std::path::Path,
    _config: &LocalConfig,
) -> Result<()> {
    let repo_path_buf = repo_path.to_path_buf();
    eprintln!(
        "\n⚠️  This will remove '{}' from ALL machines.",
        app_name
    );
    eprintln!("   • Deletes apps/{}/  from all machine branches and main", app_name);
    eprintln!("   • Removes the app from sync-rules.toml");
    eprintln!("   Note: local config files on each machine are NOT deleted.");

    if !confirm_operation(&format!("Remove '{}' from all machines?", app_name), false)? {
        println!("Cancelled.");
        return Ok(());
    }

    // Get all machine branches
    let registry = MachineRegistry::load(&repo_path_buf)?;

    // Remove from each machine branch
    for machine_id in registry.machines.keys() {
        let machine_branch = format!("machines/{}", machine_id);
        if checkout_branch(&repo_path_buf, &machine_branch).is_ok() {
            let app_dir = repo_path.join("apps").join(app_name);
            if app_dir.exists() {
                std::fs::remove_dir_all(&app_dir)?;
                commit_and_push(
                    &repo_path_buf,
                    &format!("remove {} app from {}", app_name, machine_id),
                )?;
                println!("  ✓ Removed from branch '{}'", machine_branch);
            }
        }
    }

    // Remove from main
    checkout_branch(&repo_path_buf, "main")?;

    // Delete apps/<app>/ from main if it exists
    let app_dir = repo_path.join("apps").join(app_name);
    if app_dir.exists() {
        std::fs::remove_dir_all(&app_dir)?;
    }

    // Remove from sync-rules
    let mut rules = SyncRules::load(&repo_path_buf)?;
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
