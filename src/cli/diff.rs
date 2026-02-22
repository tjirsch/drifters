use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::Result;
use crate::git::EphemeralRepoGuard;
use crate::merge::intelligent_merge;
use crate::parser::sections::{detect_comment_syntax, merge_synced_content};
use std::collections::HashMap;
use std::fs;

pub fn show_diff(app_name: Option<String>) -> Result<()> {
    log::info!("Showing diff");

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo
    println!("Fetching latest from repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Guard: detect stale machine IDs (caused by rename-machine / remove-machine
    // run from another machine while this machine was offline).
    crate::cli::common::verify_machine_registration(&config, repo_path)?;

    // Load sync rules
    let rules = SyncRules::load(repo_path)?;

    if rules.apps.is_empty() {
        println!("No apps configured for sync.");
        return Ok(());
    }

    // Determine which apps to diff
    let apps_to_diff: Vec<_> = if let Some(name) = app_name {
        if rules.apps.contains_key(&name) {
            vec![name]
        } else {
            return Err(crate::error::DriftersError::AppNotFound(name));
        }
    } else {
        rules.apps.keys().cloned().collect()
    };

    let mut total_changes = 0;

    for app in &apps_to_diff {
        let app_config = rules.apps.get(app).unwrap();

        println!("\n{}", "=".repeat(60));
        println!("App: {}", app);
        println!("{}", "=".repeat(60));

        // Resolve fileset for THIS machine
        let fileset = resolve_fileset(
            app_config,
            &config.machine_id,
            std::env::consts::OS,
        )?;

        if fileset.is_empty() {
            println!("  (no files in fileset for this machine)");
            continue;
        }

        for local_path in fileset {
            let filename = local_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            // Collect all machines' versions
            let machines_dir = repo_path
                .join("apps")
                .join(app)
                .join("machines");

            if !machines_dir.exists() {
                continue;
            }

            let all_versions = collect_machine_versions(&machines_dir, filename)?;

            if all_versions.is_empty() {
                continue;
            }

            // Intelligent merge from all machine versions
            let merged_content = intelligent_merge(
                &all_versions,
                &config.machine_id,
                filename,
                app_config,
            )?;

            // Compare with local
            let local_content = if local_path.exists() {
                fs::read_to_string(&local_path)?
            } else {
                String::new()
            };

            // Apply section merging if needed
            let final_content = if !local_content.is_empty() {
                let comment = detect_comment_syntax(filename);
                merge_synced_content(&local_content, &merged_content, comment)?
            } else {
                merged_content
            };

            // Show diff if different
            if local_content != final_content {
                println!("\n{}", filename);
                println!("{}", "-".repeat(60));
                show_file_diff(&local_content, &final_content);
                total_changes += 1;
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    if total_changes == 0 {
        println!("All configs are up to date");
    } else {
        println!("{} file(s) would change", total_changes);
        println!("\nRun 'drifters pull' to apply these changes");
    }

    Ok(())
}

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

fn show_file_diff(old: &str, new: &str) {
    use similar::TextDiff;

    let diff = TextDiff::from_lines(old, new);
    let mut shown_lines = 0;
    const MAX_LINES: usize = 100;

    for change in diff.iter_all_changes() {
        if shown_lines >= MAX_LINES {
            println!("  ... (more changes, {} lines total)", diff.iter_all_changes().count());
            break;
        }

        match change.tag() {
            similar::ChangeTag::Delete => {
                print!("  \x1b[31m-{}\x1b[0m", change);
                shown_lines += 1;
            }
            similar::ChangeTag::Insert => {
                print!("  \x1b[32m+{}\x1b[0m", change);
                shown_lines += 1;
            }
            similar::ChangeTag::Equal => {
                // Only show context lines (3 before and after)
                if shown_lines > 0 && shown_lines < MAX_LINES - 3 {
                    print!("   {}", change);
                }
            }
        }
    }
}
