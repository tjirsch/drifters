use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{confirm_operation, read_app_files, EphemeralRepoGuard};
use crate::parser::sections::{detect_comment_syntax, merge_synced_content};
use std::fs;

pub fn pull_command(
    app_name: Option<String>,
    dry_run: bool,
    from: Option<String>,
) -> Result<()> {
    log::info!("Pulling configs (dry_run: {}, from: {:?})", dry_run, from);

    // Load local config
    let config = LocalConfig::load()?;

    // Determine source branch
    let source_branch = match &from {
        Some(machine) => format!("machines/{}", machine),
        None => "main".to_string(),
    };

    // Set up ephemeral repo on the source branch
    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new_on_branch(&config, &source_branch)?;
    let repo_path = repo_guard.path();

    // Guard: detect stale machine IDs (only relevant when pulling from main;
    // --from pulls from a specific machine branch where machines.toml may not exist)
    if from.is_none() {
        crate::cli::common::verify_machine_registration(&config, repo_path)?;
    }

    // Load sync rules (from main via git show, since rules always live on main)
    let rules = load_rules_from_branch(repo_path, "main")?;

    if rules.apps.is_empty() {
        println!("No apps configured for sync.");
        return Ok(());
    }

    // Determine which apps to pull
    let pull_all = app_name.is_none();
    let apps_to_pull: Vec<_> = if let Some(name) = app_name {
        if rules.apps.contains_key(&name) {
            vec![name]
        } else {
            return Err(DriftersError::AppNotFound(name));
        }
    } else {
        rules.apps.keys().cloned().collect()
    };

    if dry_run {
        println!("(Dry run - no changes will be applied)");
    }

    println!("Pulling from branch '{}'...", source_branch);

    let mut pulled_files = 0;
    let mut warnings = Vec::new();

    for app in &apps_to_pull {
        let app_config = rules.apps.get(app).unwrap();

        println!("\nPulling configs for '{}'...", app);

        // Resolve fileset for THIS machine using current OS
        let fileset = resolve_fileset(
            app_config,
            &config.machine_id,
            std::env::consts::OS,
        )?;

        if fileset.is_empty() {
            log::warn!("No files in fileset for app '{}'", app);
            warnings.push(format!("No files in fileset for app '{}'", app));
            continue;
        }

        // In pull-all mode, skip apps that have no files present locally
        if pull_all && !fileset.iter().any(|p| p.exists()) {
            println!("  Skipping '{}': no local files found on this machine", app);
            continue;
        }

        // Read app files from the source branch
        let remote_files = read_app_files(repo_path, app)?;

        for local_path in fileset {
            let filename = local_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            // Look up this file in the remote branch's app directory
            let remote_content = match remote_files.get(filename) {
                Some(content) => content.clone(),
                None => {
                    log::debug!("No remote version for {}", filename);
                    continue;
                }
            };

            // If file exists locally, merge sections if needed
            let final_content = if local_path.exists() {
                let local_content = fs::read_to_string(&local_path)?;

                // Merge: preserve local exclude sections, update everything else
                let comment = detect_comment_syntax(filename);
                let merged_with_local = merge_synced_content(
                    &local_content,
                    &remote_content,
                    comment,
                )?;

                if merged_with_local == local_content {
                    log::debug!("{} is up to date", filename);
                    None
                } else if dry_run {
                    println!("\n  Changes in {} ({}):", filename, local_path.display());
                    show_simple_diff(&local_content, &merged_with_local);
                    println!("    (dry-run: would apply)");
                    pulled_files += 1;
                    None
                } else {
                    // Show diff and ask for confirmation
                    println!("\n  Changes in {} ({}):", filename, local_path.display());
                    show_simple_diff(&local_content, &merged_with_local);
                    let msg = format!("Apply changes to {}?", filename);
                    if confirm_operation(&msg, true)? {
                        Some(merged_with_local)
                    } else {
                        None
                    }
                }
            } else {
                // File doesn't exist locally - create it
                if dry_run {
                    println!("  {} ({}) - would be created from remote", filename, local_path.display());
                    pulled_files += 1;
                    None
                } else {
                    let msg = format!("Create {} from remote?", filename);
                    if confirm_operation(&msg, true)? {
                        Some(remote_content)
                    } else {
                        None
                    }
                }
            };

            if let Some(content) = final_content {
                // Create parent directories if needed
                if let Some(parent) = local_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(&local_path, content)?;
                println!("  ✓ {} ({})", filename, local_path.display());
                pulled_files += 1;
            } else if !dry_run {
                log::debug!("Skipped {}", filename);
            }
        }
    }

    if pulled_files == 0 && warnings.is_empty() {
        println!("\nAll configs are up to date");
        return Ok(());
    }

    // Show warnings
    if !warnings.is_empty() {
        println!("\nWarnings:");
        for warning in warnings {
            println!("  ! {}", warning);
        }
    }

    if pulled_files > 0 {
        if dry_run {
            println!(
                "\nDry run complete. {} file(s) would change.",
                pulled_files
            );
        } else {
            println!("\n✓ Successfully pulled {} file(s)", pulled_files);
        }
    }

    Ok(())
}

/// Load sync-rules.toml from a specific branch via git show.
fn load_rules_from_branch(
    repo_path: &std::path::Path,
    branch: &str,
) -> Result<SyncRules> {
    let spec = format!("{}:.drifters/sync-rules.toml", branch);
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["show", &spec])
        .output()?;

    if !output.status.success() {
        // Fallback: try reading from the current branch
        return SyncRules::load(&repo_path.to_path_buf());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let rules: SyncRules = toml::from_str(&content)?;
    Ok(rules)
}

/// Show a simple diff between two strings.
fn show_simple_diff(old: &str, new: &str) {
    use similar::TextDiff;

    let diff = TextDiff::from_lines(old, new);

    let changed_lines: Vec<_> = diff
        .iter_all_changes()
        .filter(|c| c.tag() != similar::ChangeTag::Equal)
        .collect();

    if changed_lines.is_empty() {
        println!("    (no changes)");
        return;
    }

    for change in &changed_lines {
        match change.tag() {
            similar::ChangeTag::Delete => print!("    - {}", change),
            similar::ChangeTag::Insert => print!("    + {}", change),
            similar::ChangeTag::Equal => {}
        }
    }
}
