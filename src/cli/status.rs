use crate::config::{LocalConfig, SyncRules};
use crate::error::Result;
use crate::git::EphemeralRepoGuard;
use std::fs;
use std::path::PathBuf;

pub fn show_status() -> Result<()> {
    log::info!("Showing status");

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo
    println!("Fetching latest sync rules...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Load sync rules
    let rules = SyncRules::load(repo_path)?;

    if rules.apps.is_empty() {
        println!("No apps configured for sync.");
        println!("\nUse 'drifters add <app>' to add apps");
        return Ok(());
    }

    println!("\nDrifters Status");
    println!("{}", "=".repeat(60));
    println!("Machine: {} ({})", config.machine_id, std::env::consts::OS);
    println!("Repository: {}", config.repo_url);
    println!("{}", "=".repeat(60));

    for (app_name, app_config) in &rules.apps {
        println!("\n{} ({})", app_name, format!("{:?}", app_config.sync_mode));

        // Check if this machine has exceptions
        let exceptions = app_config
            .exceptions
            .get(&config.machine_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        for file_path in &app_config.files {
            let expanded_path = expand_tilde(file_path);
            let filename = expanded_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            // Check if excepted
            if exceptions.contains(&filename.to_string()) {
                println!("  {} - EXCLUDED on this machine", filename);
                continue;
            }

            // Check local file status
            let local_exists = expanded_path.exists();

            // Check if remote has this file
            let merged_path = repo_path
                .join("apps")
                .join(app_name)
                .join("merged")
                .join(filename);
            let remote_exists = merged_path.exists();

            match (local_exists, remote_exists) {
                (true, true) => {
                    // Compare contents
                    let local_content = fs::read(&expanded_path).unwrap_or_default();
                    let remote_content = fs::read(&merged_path).unwrap_or_default();

                    if local_content == remote_content {
                        println!("  {} - ✓ up to date", filename);
                    } else {
                        println!("  {} - ↕ differs from remote", filename);
                    }
                }
                (true, false) => {
                    println!("  {} - ↑ not yet pushed", filename);
                }
                (false, true) => {
                    println!("  {} - ↓ not pulled yet", filename);
                }
                (false, false) => {
                    println!("  {} - ⚠ missing (local and remote)", filename);
                }
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Total apps: {}", rules.apps.len());
    println!("\nLegend:");
    println!("  ✓ up to date");
    println!("  ↕ differs from remote (run 'drifters push' or 'drifters pull')");
    println!("  ↑ not yet pushed (run 'drifters push')");
    println!("  ↓ not pulled yet (run 'drifters pull')");
    println!("  ⚠ missing");

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
