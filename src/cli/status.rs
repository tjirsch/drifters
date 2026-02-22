use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::Result;
use crate::git::EphemeralRepoGuard;
use std::collections::HashMap;
use std::fs;

pub fn show_status() -> Result<()> {
    log::info!("Showing status");

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo
    println!("Fetching latest sync rules...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Guard: detect stale machine IDs (caused by rename-machine / remove-machine
    // run from another machine while this machine was offline).
    crate::cli::common::verify_machine_registration(&config, repo_path)?;

    // Load sync rules
    let rules = SyncRules::load(repo_path)?;

    if rules.apps.is_empty() {
        println!("No apps configured for sync.");
        println!("\nUse 'drifters add-app <app>' to add apps");
        return Ok(());
    }

    println!("\nDrifters Status");
    println!("{}", "=".repeat(60));
    println!("Machine: {} ({})", config.machine_id, std::env::consts::OS);
    println!("Repository: {}", config.repo_url);
    println!("{}", "=".repeat(60));

    for (app_name, app_config) in &rules.apps {
        println!("\n{}", app_name);

        // Resolve fileset for this machine
        let fileset = resolve_fileset(
            app_config,
            &config.machine_id,
            std::env::consts::OS,
        )?;

        if fileset.is_empty() {
            println!("  (no files in fileset for this machine)");
            continue;
        }

        for file_path in &fileset {
            let filename = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            // Check local file status
            let local_exists = file_path.exists();

            // Collect all machine versions
            let machines_dir = repo_path
                .join("apps")
                .join(app_name)
                .join("machines");

            let all_versions = collect_machine_versions(&machines_dir, filename)?;

            // Check if this machine has pushed this file
            let this_machine_version = all_versions.get(&config.machine_id);

            match (local_exists, this_machine_version.is_some(), all_versions.is_empty()) {
                (true, true, _) => {
                    // Local exists and we've pushed it
                    let local_content = fs::read(&file_path).unwrap_or_default();
                    let remote_content = this_machine_version
                        .expect("this_machine_version is Some in (true, true, _) match arm")
                        .as_bytes();

                    if local_content == remote_content {
                        println!("  {} - ✓ up to date", filename);
                    } else {
                        println!("  {} - ↑ local changes not pushed", filename);
                    }
                }
                (true, false, false) => {
                    // Local exists but we haven't pushed, others have
                    println!("  {} - ↓ not pushed from this machine (others have versions)", filename);
                }
                (true, false, true) => {
                    // Local exists but nobody has pushed
                    println!("  {} - ↑ not yet pushed", filename);
                }
                (false, true, _) => {
                    // We've pushed but local file is missing
                    println!("  {} - ⚠ pushed from this machine but local file missing", filename);
                }
                (false, false, false) => {
                    // Local missing, we haven't pushed, but others have
                    println!("  {} - ↓ available from other machines", filename);
                }
                (false, false, true) => {
                    // Nobody has this file
                    println!("  {} - ⚠ missing (local and all remotes)", filename);
                }
            }
        }

        // Show other machines' versions if any
        let machines_with_files = list_machines_with_files(&repo_path, app_name)?;
        if !machines_with_files.is_empty() && machines_with_files.len() > 1 {
            let other_machines: Vec<_> = machines_with_files
                .into_iter()
                .filter(|m| m != &config.machine_id)
                .collect();

            if !other_machines.is_empty() {
                println!("\n  Other machines with configs: {}", other_machines.join(", "));
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Total apps: {}", rules.apps.len());
    println!("\nLegend:");
    println!("  ✓ up to date");
    println!("  ↑ local changes not pushed");
    println!("  ↓ remote changes available");
    println!("  ⚠ warning/missing");
    println!("\nRun 'drifters push-app' to sync local changes");
    println!("Run 'drifters pull-app' to get remote changes");

    Ok(())
}

/// Collect all machine versions of a specific file
fn collect_machine_versions(
    machines_dir: &std::path::Path,
    filename: &str,
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

/// List all machines that have any files for an app
fn list_machines_with_files(
    repo_path: &std::path::Path,
    app_name: &str,
) -> Result<Vec<String>> {
    let machines_dir = repo_path.join("apps").join(app_name).join("machines");

    let mut machines = Vec::new();

    if !machines_dir.exists() {
        return Ok(machines);
    }

    for entry in fs::read_dir(machines_dir)? {
        let machine_dir = entry?.path();

        if !machine_dir.is_dir() {
            continue;
        }

        if let Some(machine_id) = machine_dir.file_name().and_then(|s| s.to_str()) {
            machines.push(machine_id.to_string());
        }
    }

    Ok(machines)
}
