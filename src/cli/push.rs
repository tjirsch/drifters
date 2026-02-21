use crate::config::{LocalConfig, SyncMode, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{check_file_safety, commit_and_push, confirm_operation, EphemeralRepoGuard};
use crate::parser::markers::{detect_comment_syntax, extract_synced_content};
use std::fs;
use std::path::PathBuf;

pub fn push_command(app_name: Option<String>, yolo: bool) -> Result<()> {
    log::info!("Pushing configs (yolo: {})", yolo);

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo (clones/pulls automatically, cleans up on drop)
    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

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

        // Check if this machine has exceptions for this app
        let exceptions = app_config
            .exceptions
            .get(&config.machine_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        for file_path in &app_config.files {
            // Expand home directory
            let expanded_path = expand_tilde(file_path);

            // Get filename
            let filename = expanded_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            // Check if excepted for this machine
            if exceptions.contains(&filename.to_string()) {
                log::debug!("Skipping {} (excepted for {})", filename, config.machine_id);
                continue;
            }

            if !expanded_path.exists() {
                log::warn!("File not found: {:?}", expanded_path);
                warnings.push(format!("File not found: {:?}", expanded_path));
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
                if !check_file_safety(&expanded_path, &dest_path)? {
                    let msg = format!(
                        "File {:?} appears risky to push. Continue?",
                        expanded_path
                    );
                    if !confirm_operation(&msg, false)? {
                        log::info!("Skipped {}", filename);
                        continue;
                    }
                }
            }

            // Read file content
            let content = fs::read_to_string(&expanded_path)?;

            // Handle different sync modes
            let content_to_sync = match &app_config.sync_mode {
                SyncMode::Full => content.clone(),
                SyncMode::Markers => {
                    let comment = detect_comment_syntax(filename);
                    match extract_synced_content(&content, comment)? {
                        Some(synced) => synced,
                        None => {
                            log::warn!(
                                "No sync markers found in {} (using marker mode). Skipping.",
                                filename
                            );
                            warnings.push(format!(
                                "No sync markers found in {} (add {}-start-sync- and {}-stop-sync-)",
                                filename, comment, comment
                            ));
                            continue;
                        }
                    }
                }
                _ => {
                    log::warn!("Unsupported sync mode: {:?}. Using full sync.", app_config.sync_mode);
                    content.clone()
                }
            };

            // Write to destination
            fs::write(&dest_path, &content_to_sync)?;
            log::debug!("Wrote content to {:?}", dest_path);

            // Update merged state
            let merged_dir = repo_path.join("apps").join(app).join("merged");
            fs::create_dir_all(&merged_dir)?;

            let merged_path = merged_dir.join(filename);
            fs::write(&merged_path, &content_to_sync)?;

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

fn expand_tilde(path: &PathBuf) -> PathBuf {
    if let Some(s) = path.to_str() {
        if s.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&s[2..]);
            }
        }
    }
    path.clone()
}
