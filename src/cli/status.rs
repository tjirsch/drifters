use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::Result;
use crate::git::{
    checkout_branch, list_branches, read_app_files, EphemeralRepoGuard,
};
use std::fs;

pub fn show_status() -> Result<()> {
    log::info!("Showing status");

    // Load local config
    let config = LocalConfig::load()?;
    let machine_branch = format!("machines/{}", config.machine_id);

    // Set up ephemeral repo
    println!("Fetching latest sync rules...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Guard: detect stale machine IDs
    crate::cli::common::verify_machine_registration(&config, repo_path)?;

    // Load sync rules from main
    let rules = SyncRules::load(repo_path)?;

    println!("\nDrifters Status");
    println!("{}", "=".repeat(60));
    println!("Machine: {} ({})", config.machine_id, std::env::consts::OS);
    println!("Branch:  {}", machine_branch);
    println!("Repository: {}", config.repo_url);

    // Show available branches
    let branches = list_branches(repo_path).unwrap_or_default();
    let machine_branches: Vec<_> = branches
        .iter()
        .filter(|b| b.starts_with("machines/") || b.starts_with("origin/machines/"))
        .filter(|b| !b.contains("HEAD"))
        .collect();
    if !machine_branches.is_empty() {
        println!("Machine branches: {}", machine_branches
            .iter()
            .map(|b| b.strip_prefix("origin/").unwrap_or(b))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", "));
    }

    println!("{}", "=".repeat(60));

    if rules.apps.is_empty() {
        println!("No apps configured for sync.");
        println!("\nUse 'drifters add-app <app>' to add apps");
        return Ok(());
    }

    // Check what's on this machine's branch
    let machine_files = if checkout_branch(repo_path, &machine_branch).is_ok() {
        let mut all_files = std::collections::HashMap::new();
        for app_name in rules.apps.keys() {
            let files = read_app_files(repo_path, app_name)?;
            all_files.insert(app_name.clone(), files);
        }
        // Switch back to main
        let _ = checkout_branch(repo_path, "main");
        Some(all_files)
    } else {
        None
    };

    // Check what's on main
    let main_files = {
        let _ = checkout_branch(repo_path, "main");
        let mut all_files = std::collections::HashMap::new();
        for app_name in rules.apps.keys() {
            let files = read_app_files(repo_path, app_name)?;
            all_files.insert(app_name.clone(), files);
        }
        all_files
    };

    for (app_name, app_config) in &rules.apps {
        println!("\n{}", app_name);

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

            let local_exists = file_path.exists();
            let on_branch = machine_files
                .as_ref()
                .and_then(|mf| mf.get(app_name))
                .and_then(|files| files.get(filename));
            let on_main = main_files
                .get(app_name)
                .and_then(|files| files.get(filename));

            match (local_exists, on_branch.is_some(), on_main.is_some()) {
                (true, true, _) => {
                    let local_content = fs::read_to_string(file_path).unwrap_or_default();
                    if on_branch.map(|c| c == &local_content).unwrap_or(false) {
                        println!("  {} ({}) - ✓ up to date on branch", filename, file_path.display());
                    } else {
                        println!("  {} ({}) - ↑ local changes not pushed", filename, file_path.display());
                    }
                }
                (true, false, _) => {
                    println!("  {} ({}) - ↑ not yet pushed to branch", filename, file_path.display());
                }
                (false, _, true) => {
                    println!("  {} ({}) - ↓ available on main", filename, file_path.display());
                }
                (false, true, false) => {
                    println!("  {} ({}) - ⚠ on branch but missing locally", filename, file_path.display());
                }
                (false, false, false) => {
                    println!("  {} ({}) - ⚠ missing everywhere", filename, file_path.display());
                }
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Total apps: {}", rules.apps.len());
    println!("\nLegend:");
    println!("  ✓ up to date on branch");
    println!("  ↑ local changes not pushed");
    println!("  ↓ remote changes available");
    println!("  ⚠ warning/missing");
    println!("\nWorkflow:");
    println!("  drifters push-app    — push local changes to your machine branch");
    println!("  drifters merge-app   — merge your branch into main");
    println!("  drifters pull-app    — pull from main to local");

    Ok(())
}
