use crate::config::{LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{
    checkout_branch, commit_merge, confirm_operation, merge_branch,
    merge_dry_run, run_mergetool, EphemeralRepoGuard,
};

pub fn merge_command(
    app_name: Option<String>,
    from: Option<String>,
    dry_run: bool,
) -> Result<()> {
    log::info!("Merging machine branch into main");

    // Load config
    let local_config = LocalConfig::load()?;

    // Determine source machine
    let source_machine = from.unwrap_or_else(|| local_config.machine_id.clone());
    let source_branch = format!("machines/{}", source_machine);

    // Set up ephemeral repo on main
    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new(&local_config)?;
    let repo_path = repo_guard.path();

    // Guard: detect stale machine IDs
    crate::cli::common::verify_machine_registration(&local_config, repo_path)?;

    // Check if the source machine is singular
    let rules = SyncRules::load(repo_path)?;
    if is_singular_machine(&source_machine, &rules) {
        println!(
            "Machine '{}' is marked as singular — its branch should not be merged into main.",
            source_machine
        );
        println!("To change this, remove `singular = true` from the machine's override in sync-rules.toml.");
        return Ok(());
    }

    // If app_name is specified, we note it but git merge operates on whole branches
    if let Some(ref name) = app_name {
        if !rules.apps.contains_key(name) {
            return Err(DriftersError::AppNotFound(name.clone()));
        }
        println!("Note: git merge operates on entire branches. Merging all files from '{}'.", source_branch);
    }

    // Make sure we're on main
    checkout_branch(repo_path, "main")?;

    if dry_run {
        println!("(Dry run - showing what would change)");
        match merge_dry_run(repo_path, &source_branch) {
            Ok((clean, diff)) => {
                if diff.is_empty() {
                    println!("\nNo changes to merge from '{}'.", source_branch);
                } else if clean {
                    println!("\nClean merge from '{}':", source_branch);
                    println!("{}", diff);
                } else {
                    println!("\nMerge from '{}' would have conflicts:", source_branch);
                    println!("{}", diff);
                }
            }
            Err(e) => {
                println!("Could not perform dry-run merge: {}", e);
            }
        }
        return Ok(());
    }

    // Confirm
    println!(
        "\nMerge '{}' into main?",
        source_branch
    );
    if !confirm_operation("Proceed?", true)? {
        println!("Cancelled.");
        return Ok(());
    }

    // Perform the merge
    println!("\nMerging '{}' into main...", source_branch);
    match merge_branch(repo_path, &source_branch) {
        Ok(()) => {
            println!("✓ Clean merge — no conflicts.");
        }
        Err(DriftersError::MergeConflict(msg)) => {
            println!("\n⚠️  Merge conflicts detected:");
            println!("{}", msg);
            println!("\nLaunching mergetool to resolve conflicts...");

            run_mergetool(repo_path)?;

            // After mergetool, commit the resolution
            let merge_msg = format!(
                "Merge {} into main (conflicts resolved)",
                source_branch
            );
            commit_merge(repo_path, &merge_msg)?;
            println!("✓ Conflicts resolved and committed.");
        }
        Err(e) => return Err(e),
    }

    // Push main
    println!("\nPushing main...");
    // Use commit_and_push which handles the push part
    // But we already committed via merge, so we just need to push
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["push", "-u", "origin", "main"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(DriftersError::Git(format!(
            "Failed to push main: {}", stderr
        )));
    }

    println!("✓ Successfully merged '{}' into main.", source_branch);

    Ok(())
}

/// Check if a machine is marked as singular in sync-rules.
fn is_singular_machine(machine_id: &str, rules: &SyncRules) -> bool {
    for app_config in rules.apps.values() {
        if let Some(override_config) = app_config.machines.get(machine_id) {
            if override_config.singular {
                return true;
            }
        }
    }
    false
}
