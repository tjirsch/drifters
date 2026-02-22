use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{confirm_operation, EphemeralRepoGuard};
use crate::merge::intelligent_merge;
use crate::parser::sections::{detect_comment_syntax, merge_synced_content};
use std::collections::HashMap;
use std::fs;

pub fn pull_command(app_name: Option<String>, yolo: bool) -> Result<()> {
    log::info!("Pulling configs (yolo: {})", yolo);

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo (clones/pulls automatically, cleans up on drop)
    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Load sync rules (may have been updated by other machines)
    let rules = SyncRules::load(repo_path)?;

    if rules.apps.is_empty() {
        println!("No apps configured for sync.");
        return Ok(());
    }

    // Determine which apps to pull
    let apps_to_pull: Vec<_> = if let Some(name) = app_name {
        if rules.apps.contains_key(&name) {
            vec![name]
        } else {
            return Err(DriftersError::AppNotFound(name));
        }
    } else {
        rules.apps.keys().cloned().collect()
    };

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

        for local_path in fileset {
            // Get filename
            let filename = local_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            // Collect all machines' versions of this file
            let machines_dir = repo_path
                .join("apps")
                .join(app)
                .join("machines");

            if !machines_dir.exists() {
                log::debug!("No machines directory for app '{}'", app);
                warnings.push(format!("No machine configs for app '{}'", app));
                continue;
            }

            let mut all_versions = collect_machine_versions(&machines_dir, filename)?;

            // Include the current machine's local file in the consensus if it
            // has not yet been pushed (i.e. no repo entry for this machine ID).
            // Without this, local edits made since the last `drifters push`
            // would be invisible to the vote and could be overwritten.
            if local_path.exists() && !all_versions.contains_key(&config.machine_id) {
                match fs::read_to_string(&local_path) {
                    Ok(local_content) => {
                        log::debug!(
                            "{}: local version added to consensus (not yet pushed)",
                            filename
                        );
                        all_versions.insert(config.machine_id.clone(), local_content);
                    }
                    Err(e) => {
                        log::warn!("Could not read local file {:?}: {}", local_path, e);
                    }
                }
            }

            if all_versions.is_empty() {
                log::debug!("No versions found for {}", filename);
                warnings.push(format!("No remote versions for: {}", filename));
                continue;
            }

            // Intelligent merge from all machine versions
            let merged_content = intelligent_merge(
                &all_versions,
                &config.machine_id,
                filename,
                app_config,
            )?;

            // If file exists locally, merge sections if needed
            let final_content = if local_path.exists() {
                let local_content = fs::read_to_string(&local_path)?;

                // Merge: preserve local exclude sections, update everything else
                let comment = detect_comment_syntax(filename);
                let merged_with_local = merge_synced_content(
                    &local_content,
                    &merged_content,
                    comment,
                )?;

                if merged_with_local == local_content {
                    log::debug!("{} is up to date", filename);
                    None
                } else if !yolo {
                    // Show diff and ask for confirmation
                    println!("\n  Changes in {}:", filename);
                    show_simple_diff(&local_content, &merged_with_local);
                    let msg = format!("Apply changes to {}?", filename);
                    if confirm_operation(&msg, true)? {
                        Some(merged_with_local)
                    } else {
                        None
                    }
                } else {
                    Some(merged_with_local)
                }
            } else {
                // File doesn't exist locally - create it
                if !yolo {
                    let msg = format!("Create {} from remote?", filename);
                    if confirm_operation(&msg, true)? {
                        Some(merged_content)
                    } else {
                        None
                    }
                } else {
                    Some(merged_content)
                }
            };

            if let Some(content) = final_content {
                // Create parent directories if needed
                if let Some(parent) = local_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(&local_path, content)?;
                println!("  ✓ {}", filename);
                pulled_files += 1;
            } else {
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
        println!("\n✓ Successfully pulled {} file(s)", pulled_files);
    }

    Ok(())
}

/// Collect all machine versions of a specific file
fn collect_machine_versions(
    machines_dir: &std::path::Path,
    filename: &str,
) -> Result<HashMap<String, String>> {
    let mut versions = HashMap::new();

    for entry in fs::read_dir(machines_dir)? {
        let machine_dir = entry?.path();

        if !machine_dir.is_dir() {
            continue;
        }

        let machine_id = machine_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_path = machine_dir.join(filename);
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            versions.insert(machine_id, content);
        }
    }

    Ok(versions)
}

/// Show a simple diff between two strings
fn show_simple_diff(old: &str, new: &str) {
    use similar::TextDiff;

    let diff = TextDiff::from_lines(old, new);
    let mut changes = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Delete => {
                print!("    - {}", change);
                changes += 1;
            }
            similar::ChangeTag::Insert => {
                print!("    + {}", change);
                changes += 1;
            }
            similar::ChangeTag::Equal => {
                // Don't print unchanged lines in summary
            }
        }

        if changes >= 10 {
            println!("    ... (more changes)");
            break;
        }
    }

    if changes == 0 {
        println!("    (no changes)");
    }
}
