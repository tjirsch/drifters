use crate::config::{LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{
    checkout_branch, checkout_paths, commit_and_push, commit_merge, confirm_operation,
    fetch_branch, merge_branch, merge_dry_run, run_mergetool, EphemeralRepoGuard,
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

    // Make sure we're on main
    checkout_branch(repo_path, "main")?;

    // Fetch the source branch so git knows about it (clone only gets main)
    fetch_branch(repo_path, &source_branch)?;
    let merge_ref = format!("origin/{}", source_branch);

    if let Some(ref name) = app_name {
        // ── Selective merge: single app ─────────────────────────────────────
        if !rules.apps.contains_key(name) {
            return Err(DriftersError::AppNotFound(name.clone()));
        }

        let pathspec = format!("apps/{}/", name);

        if dry_run {
            println!("(Dry run - showing what would change for '{}')", name);
            let diff = diff_paths(repo_path, &merge_ref, &pathspec)?;
            if diff.is_empty() {
                println!("\nNo changes to merge for '{}' from '{}'.", name, source_branch);
            } else {
                println!("\nChanges for '{}' from '{}':", name, source_branch);
                println!("{}", diff);
            }
            return Ok(());
        }

        println!(
            "\nMerge '{}' from '{}' into main?",
            name, source_branch
        );
        if !confirm_operation("Proceed?", true)? {
            println!("Cancelled.");
            return Ok(());
        }

        println!("\nMerging '{}' from '{}'...", name, source_branch);
        checkout_paths(repo_path, &merge_ref, &pathspec)?;
        commit_and_push(
            repo_path,
            &format!("Merge {} from {}", name, source_branch),
        )?;
        println!("✓ Successfully merged '{}' from '{}' into main.", name, source_branch);
    } else {
        // ── Full branch merge ───────────────────────────────────────────────
        // Check for no_merge apps
        let no_merge_apps: Vec<&String> = rules
            .apps
            .iter()
            .filter(|(_, config)| config.no_merge)
            .map(|(name, _)| name)
            .collect();

        if !no_merge_apps.is_empty() {
            let mut names: Vec<&&String> = no_merge_apps.iter().collect();
            names.sort();
            println!("The following apps are marked no_merge and will not be included:");
            for name in &names {
                println!("  - {}", name);
            }
            println!("\nTo merge them individually, run: drifters merge-app <app-name>");
            println!("To include them in full merges, remove `no_merge = true` from sync-rules.toml.\n");

            // Collect mergeable app names for selective merge
            let mergeable_apps: Vec<String> = rules
                .apps
                .iter()
                .filter(|(_, config)| !config.no_merge)
                .map(|(name, _)| name.clone())
                .collect();

            if mergeable_apps.is_empty() {
                println!("No apps to merge (all are marked no_merge).");
                return Ok(());
            }

            // Use selective merge for each mergeable app
            if dry_run {
                println!("(Dry run - showing what would change)");
                for app in &mergeable_apps {
                    let pathspec = format!("apps/{}/", app);
                    let diff = diff_paths(repo_path, &merge_ref, &pathspec)?;
                    if !diff.is_empty() {
                        println!("\nChanges for '{}':", app);
                        println!("{}", diff);
                    }
                }
                return Ok(());
            }

            println!(
                "Merge {} app(s) from '{}' into main?",
                mergeable_apps.len(),
                source_branch
            );
            if !confirm_operation("Proceed?", true)? {
                println!("Cancelled.");
                return Ok(());
            }

            println!("\nMerging selectively from '{}'...", source_branch);
            for app in &mergeable_apps {
                let pathspec = format!("apps/{}/", app);
                checkout_paths(repo_path, &merge_ref, &pathspec)?;
            }
            commit_and_push(
                repo_path,
                &format!("Merge {} app(s) from {} (excluding no_merge)", mergeable_apps.len(), source_branch),
            )?;
            println!("✓ Successfully merged {} app(s) into main.", mergeable_apps.len());
        } else {
            // No no_merge apps — full git merge
            if dry_run {
                println!("(Dry run - showing what would change)");
                match merge_dry_run(repo_path, &merge_ref) {
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

            println!(
                "\nMerge '{}' into main?",
                source_branch
            );
            if !confirm_operation("Proceed?", true)? {
                println!("Cancelled.");
                return Ok(());
            }

            println!("\nMerging '{}' into main...", source_branch);
            match merge_branch(repo_path, &merge_ref) {
                Ok(()) => {
                    println!("✓ Clean merge — no conflicts.");
                }
                Err(DriftersError::MergeConflict(msg)) => {
                    println!("\n⚠️  Merge conflicts detected:");
                    println!("{}", msg);
                    println!("\nLaunching mergetool to resolve conflicts...");

                    run_mergetool(repo_path)?;

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
        }
    }

    Ok(())
}

/// Show diff of specific paths between main and a ref.
fn diff_paths(
    repo_path: &std::path::Path,
    source_ref: &str,
    pathspec: &str,
) -> Result<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["diff", "HEAD", source_ref, "--stat", "--", pathspec])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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
