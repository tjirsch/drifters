use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::Result;
use crate::git::EphemeralRepoGuard;
use crate::merge::intelligent_merge;
use crate::parser::sections::{detect_comment_syntax, merge_synced_content};
use std::collections::HashMap;
use std::fs;

pub fn merge_command(
    app_name: Option<String>,
    filter_machine: Option<String>,
    filter_os: Option<String>,
    dry_run: bool,
    yolo: bool,
) -> Result<()> {
    log::info!("Running merge with current rules");

    // Load config
    let local_config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&local_config)?;
    let repo_path = repo_guard.path();

    // Load sync rules (potentially updated)
    let sync_rules = SyncRules::load(repo_path)?;

    // Determine which apps to merge
    let apps_to_merge: Vec<String> = match app_name {
        Some(name) => {
            if !sync_rules.apps.contains_key(&name) {
                return Err(crate::error::DriftersError::AppNotFound(name));
            }
            vec![name]
        }
        None => sync_rules.apps.keys().cloned().collect(),
    };

    println!("Re-merging with current rules...");
    if let Some(ref machine) = filter_machine {
        println!("Considering only machine: {}", machine);
    }
    if let Some(ref os) = filter_os {
        println!("Using OS rules for: {}", os);
    }
    if dry_run {
        println!("(Dry run - no changes will be applied)");
    }

    let mut total_changes = 0;

    // For each app
    for app in apps_to_merge {
        println!("\n{}", "=".repeat(60));
        println!("App: {}", app);

        let app_config = sync_rules
            .apps
            .get(&app)
            .ok_or_else(|| crate::error::DriftersError::AppNotFound(app.clone()))?;

        // Resolve fileset using CURRENT rules
        let target_os = filter_os
            .as_deref()
            .unwrap_or(std::env::consts::OS);
        let fileset = resolve_fileset(app_config, &local_config.machine_id, target_os)?;

        println!("Files in fileset: {}", fileset.len());

        // For each file in fileset
        for local_path in fileset {
            let filename = local_path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| crate::error::DriftersError::Config("Invalid filename".to_string()))?;

            // Collect machine versions (filtered if requested)
            let machines_dir = repo_path.join("apps").join(&app).join("machines");

            if !machines_dir.exists() {
                log::debug!("No machines directory for app '{}'", app);
                continue;
            }

            let all_versions =
                collect_machine_versions(&machines_dir, filename, filter_machine.as_deref())?;

            if all_versions.is_empty() {
                log::debug!("No versions found for {}", filename);
                continue;
            }

            println!(
                "\n  {} (from {} machine{})",
                filename,
                all_versions.len(),
                if all_versions.len() == 1 { "" } else { "s" }
            );

            // Read current local state
            let current_local = if local_path.exists() {
                Some(fs::read_to_string(&local_path)?)
            } else {
                None
            };

            // Run intelligent merge with CURRENT rules
            let merged_content =
                intelligent_merge(&all_versions, &local_config.machine_id, filename, app_config)?;

            // Apply section merging if needed
            let final_content = if let Some(ref local) = current_local {
                let comment = detect_comment_syntax(filename);
                merge_synced_content(local, &merged_content, comment)?
            } else {
                merged_content
            };

            // Compare with current local
            if let Some(ref local) = current_local {
                if local == &final_content {
                    println!("    No changes");
                    continue;
                }
            }

            // Show diff
            if !yolo {
                show_file_diff(
                    filename,
                    current_local.as_deref().unwrap_or(""),
                    &final_content,
                )?;
            }

            total_changes += 1;

            // Apply if not dry-run
            if !dry_run {
                if !yolo {
                    if !confirm(&format!("Apply changes to {}?", filename))? {
                        continue;
                    }
                }

                // Create parent directories if needed
                if let Some(parent) = local_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(&local_path, final_content)?;
                println!("    ✓ Applied");
            } else {
                println!("    (dry-run: would apply)");
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    if dry_run {
        println!(
            "Dry run complete. {} file{} would change.",
            total_changes,
            if total_changes == 1 { "" } else { "s" }
        );
    } else {
        println!(
            "Merge complete. {} file{} updated.",
            total_changes,
            if total_changes == 1 { "" } else { "s" }
        );
    }

    Ok(())
}

/// Collect machine versions, optionally filtered by machine ID
fn collect_machine_versions(
    machines_dir: &std::path::Path,
    filename: &str,
    filter_machine: Option<&str>,
) -> Result<HashMap<String, String>> {
    let mut versions = HashMap::new();

    if !machines_dir.exists() {
        return Ok(versions);
    }

    for entry in fs::read_dir(machines_dir)? {
        let machine_dir = entry?.path();

        if !machine_dir.is_dir() {
            continue;
        }

        let machine_id = machine_dir
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| crate::error::DriftersError::Config("Invalid machine dir".to_string()))?
            .to_string();

        // Apply filter if specified
        if let Some(filter) = filter_machine {
            if machine_id != filter {
                continue;
            }
        }

        let file_path = machine_dir.join(filename);
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            versions.insert(machine_id, content);
        }
    }

    Ok(versions)
}

/// Show diff for a file
fn show_file_diff(_filename: &str, old: &str, new: &str) -> Result<()> {
    use similar::TextDiff;

    let diff = TextDiff::from_lines(old, new);

    println!("    Changes:");
    let mut changes = 0;
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal => " ",
        };
        print!("      {}{}", sign, change);
        if sign != " " {
            changes += 1;
        }
        if changes >= 20 {
            println!("      ... (more changes)");
            break;
        }
    }

    Ok(())
}

/// Simple confirmation prompt
fn confirm(msg: &str) -> Result<bool> {
    use std::io::{self, Write};

    print!("{} [y/N] ", msg);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}
