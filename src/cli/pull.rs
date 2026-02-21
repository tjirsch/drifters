use crate::config::{LocalConfig, SyncMode, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{confirm_operation, EphemeralRepoGuard};
use crate::parser::markers::{detect_comment_syntax, insert_synced_content};
use std::fs;
use std::path::PathBuf;

pub fn pull_command(app_name: Option<String>, yolo: bool) -> Result<()> {
    log::info!("Pulling configs (yolo: {})", yolo);

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

            // Source in repo: apps/[app]/merged/[filename]
            let source_path = repo_path
                .join("apps")
                .join(app)
                .join("merged")
                .join(filename);

            if !source_path.exists() {
                log::debug!("Merged file not found: {:?}", source_path);
                warnings.push(format!("No merged config for: {}", filename));
                continue;
            }

            // Create parent directories if needed
            if let Some(parent) = expanded_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Read remote content
            let remote_content = fs::read_to_string(&source_path)?;

            // Determine final content based on sync mode
            let final_content = if expanded_path.exists() {
                let local_content = fs::read_to_string(&expanded_path)?;

                match &app_config.sync_mode {
                    SyncMode::Full => {
                        // Full mode: replace entire file
                        if local_content == remote_content {
                            log::debug!("{} is up to date", filename);
                            None
                        } else if !yolo {
                            let msg = format!("Overwrite local {:?} with remote version?", filename);
                            if confirm_operation(&msg, true)? {
                                Some(remote_content)
                            } else {
                                None
                            }
                        } else {
                            Some(remote_content)
                        }
                    }
                    SyncMode::Markers => {
                        // Markers mode: replace only synced sections
                        let comment = detect_comment_syntax(filename);
                        let merged = insert_synced_content(&local_content, &remote_content, comment)?;

                        if merged == local_content {
                            log::debug!("{} is up to date", filename);
                            None
                        } else if !yolo {
                            let msg = format!("Update synced sections in {:?}?", filename);
                            if confirm_operation(&msg, true)? {
                                Some(merged)
                            } else {
                                None
                            }
                        } else {
                            Some(merged)
                        }
                    }
                    _ => {
                        log::warn!("Unsupported sync mode: {:?}. Using full sync.", app_config.sync_mode);
                        if local_content != remote_content && !yolo {
                            let msg = format!("Overwrite local {:?} with remote version?", filename);
                            if confirm_operation(&msg, true)? {
                                Some(remote_content)
                            } else {
                                None
                            }
                        } else if local_content != remote_content {
                            Some(remote_content)
                        } else {
                            None
                        }
                    }
                }
            } else {
                // File doesn't exist locally - create it
                if !yolo {
                    let msg = format!("Create {:?} from remote?", filename);
                    if confirm_operation(&msg, true)? {
                        Some(remote_content)
                    } else {
                        None
                    }
                } else {
                    Some(remote_content)
                }
            };

            if let Some(content) = final_content {
                fs::write(&expanded_path, content)?;
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
