use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{check_file_safety, commit_and_push, confirm_operation, EphemeralRepoGuard};
use crate::parser::sections::{detect_comment_syntax, extract_syncable_content};
use std::fs;

pub fn push_command(app_name: Option<String>, yolo: bool) -> Result<()> {
    log::info!("Pushing configs (yolo: {})", yolo);

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo (clones/pulls automatically, cleans up on drop)
    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Guard: detect stale machine IDs (caused by rename-machine / remove-machine
    // run from another machine while this machine was offline).
    crate::cli::common::verify_machine_registration(&config, repo_path)?;

    // Load sync rules
    let rules = SyncRules::load(repo_path)?;

    if rules.apps.is_empty() {
        println!("No apps configured for sync.");
        println!("Use 'drifters add <app>' to add apps");
        return Ok(());
    }

    // Determine which apps to push
    let apps_to_push: Vec<_> = if let Some(name) = app_name {
        if rules.apps.contains_key(&name) {
            vec![name]
        } else {
            return Err(DriftersError::AppNotFound(name));
        }
    } else {
        rules.apps.keys().cloned().collect()
    };

    let mut pushed_files = 0;
    let mut warnings = Vec::new();

    for app in &apps_to_push {
        let app_config = rules.apps.get(app).unwrap();

        println!("\nPushing configs for '{}'...", app);

        // Resolve fileset for this machine using current OS
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

        for file_path in fileset {
            // Get filename
            let filename = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            if !file_path.exists() {
                log::warn!("File not found: {:?}", file_path);
                warnings.push(format!("File not found: {:?}", file_path));
                continue;
            }

            // Destination in repo: apps/[app]/machines/[machine-id]/[filename]
            let dest_dir = repo_path
                .join("apps")
                .join(app)
                .join("machines")
                .join(&config.machine_id);

            fs::create_dir_all(&dest_dir)?;

            let dest_path = dest_dir.join(filename);

            // Safety check (unless --yolo)
            if !yolo {
                if !check_file_safety(&file_path, &dest_path)? {
                    let msg = format!(
                        "File {:?} appears risky to push. Continue?",
                        file_path
                    );
                    if !confirm_operation(&msg, false)? {
                        log::info!("Skipped {}", filename);
                        continue;
                    }
                }
            }

            // Read file content
            let content = fs::read_to_string(&file_path)?;

            // Try to extract syncable content (excludes drifters::exclude sections)
            let comment = detect_comment_syntax(filename);
            let content_to_sync = match extract_syncable_content(&content, comment)? {
                Some(syncable) => {
                    log::debug!("Found section tags in {}, syncing non-excluded content", filename);
                    syncable
                }
                None => {
                    // No tags found, sync entire file
                    log::debug!("No section tags in {}, syncing entire file", filename);
                    content.clone()
                }
            };

            // Write to machines/[machine-id]/ only
            fs::write(&dest_path, &content_to_sync)?;
            log::debug!("Wrote content to {:?}", dest_path);

            println!("  ✓ {}", filename);
            pushed_files += 1;
        }
    }

    if pushed_files == 0 {
        println!("\nNo files to push");
        return Ok(());
    }

    // Show warnings
    if !warnings.is_empty() {
        println!("\nWarnings:");
        for warning in warnings {
            println!("  ! {}", warning);
        }
    }

    // Confirm push
    if !yolo {
        println!("\nPushed {} file(s) for {} app(s)", pushed_files, apps_to_push.len());
        if !confirm_operation("Commit and push these changes?", true)? {
            return Err(DriftersError::UserCancelled);
        }
    }

    // Commit and push
    println!("\nCommitting changes...");
    let message = if apps_to_push.len() == 1 {
        format!("Update {} configs from {}", apps_to_push[0], config.machine_id)
    } else {
        format!("Update configs from {}", config.machine_id)
    };

    commit_and_push(repo_path, &message)?;

    println!("✓ Successfully pushed {} file(s)", pushed_files);

    Ok(())
}
