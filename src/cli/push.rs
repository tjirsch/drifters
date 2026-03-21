use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{check_file_safety, commit_and_push, confirm_operation, EphemeralRepoGuard};
use crate::parser::sections::{detect_comment_syntax, extract_syncable_content};
use std::fs;

pub fn push_command(app_name: Option<String>) -> Result<()> {
    log::info!("Pushing configs to machine branch");

    // Load local config
    let config = LocalConfig::load()?;
    let machine_branch = format!("machines/{}", config.machine_id);

    // Set up ephemeral repo on this machine's branch
    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new_on_branch(&config, &machine_branch)?;
    let repo_path = repo_guard.path();

    // Guard: detect stale machine IDs
    crate::cli::common::verify_machine_registration(&config, repo_path)?;

    // Load sync rules from main (checkout main temporarily to read rules, then switch back)
    // sync-rules.toml lives on main, so we read it via git show
    let rules = load_rules_from_main(repo_path)?;

    if rules.apps.is_empty() {
        println!("No apps configured for sync.");
        println!("Use 'drifters add-app <app>' to add apps");
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

            // Destination in repo: apps/[app]/[filename] (flat, on machine branch)
            let dest_dir = repo_path
                .join("apps")
                .join(app);

            fs::create_dir_all(&dest_dir)?;

            let dest_path = dest_dir.join(filename);

            // Safety check
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

            // Write to apps/[app]/[filename] on machine branch
            fs::write(&dest_path, &content_to_sync)?;
            log::debug!("Wrote content to {:?}", dest_path);

            println!("  ✓ {} ({})", filename, file_path.display());
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
    println!("\nPushed {} file(s) for {} app(s) to branch '{}'", pushed_files, apps_to_push.len(), machine_branch);
    if !confirm_operation("Commit and push these changes?", true)? {
        return Err(DriftersError::UserCancelled);
    }

    // Commit and push
    println!("\nCommitting changes...");
    let message = if apps_to_push.len() == 1 {
        format!("Update {} configs from {}", apps_to_push[0], config.machine_id)
    } else {
        format!("Update configs from {}", config.machine_id)
    };

    commit_and_push(repo_path, &message)?;

    println!("✓ Successfully pushed {} file(s) to branch '{}'", pushed_files, machine_branch);

    Ok(())
}

/// Load sync-rules.toml from the main branch without switching branches.
/// Uses `git show main:.drifters/sync-rules.toml`.
fn load_rules_from_main(repo_path: &std::path::Path) -> Result<SyncRules> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["show", "main:.drifters/sync-rules.toml"])
        .output()?;

    if !output.status.success() {
        // Fallback: try reading from the current branch (for new repos where main
        // might have the file from init)
        return SyncRules::load(&repo_path.to_path_buf());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let rules: SyncRules = toml::from_str(&content)?;
    Ok(rules)
}
