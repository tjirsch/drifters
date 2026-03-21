use crate::config::{LocalConfig, MachineRegistry};
use crate::error::Result;
use crate::git::{
    checkout_branch, commit_and_push, confirm_operation, create_branch, EphemeralRepoGuard,
};
use std::collections::HashMap;
use std::fs;

/// Migrate an existing repo from the old `apps/<app>/machines/<id>/` directory
/// layout to the new branch-per-machine model.
///
/// For each machine found in the old layout:
/// 1. Creates a branch `machines/<machine_id>` from main
/// 2. Copies files to flat `apps/<app>/<filename>` layout on that branch
/// 3. Commits and pushes the branch
///
/// Then cleans up the old `machines/` directories from main.
pub fn migrate() -> Result<()> {
    log::info!("Migrating repo to branch-per-machine layout");

    let config = LocalConfig::load()?;

    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Load registry to know which machines exist
    let mut registry = MachineRegistry::load(repo_path)?;

    // Scan the old layout: apps/<app>/machines/<id>/<files>
    let apps_dir = repo_path.join("apps");
    if !apps_dir.exists() {
        println!("No apps directory found. Nothing to migrate.");
        return Ok(());
    }

    // Collect old layout data: machine_id -> app_name -> filename -> content
    let mut machine_data: HashMap<String, HashMap<String, HashMap<String, String>>> = HashMap::new();
    let mut found_old_layout = false;

    for app_entry in fs::read_dir(&apps_dir)? {
        let app_dir = app_entry?.path();
        if !app_dir.is_dir() {
            continue;
        }

        let app_name = app_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let machines_dir = app_dir.join("machines");
        if !machines_dir.exists() {
            continue;
        }

        found_old_layout = true;

        for machine_entry in fs::read_dir(&machines_dir)? {
            let machine_dir = machine_entry?.path();
            if !machine_dir.is_dir() {
                continue;
            }

            let machine_id = machine_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            for file_entry in fs::read_dir(&machine_dir)? {
                let file_path = file_entry?.path();
                if !file_path.is_file() {
                    continue;
                }

                let filename = file_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let content = fs::read_to_string(&file_path)?;

                machine_data
                    .entry(machine_id.clone())
                    .or_default()
                    .entry(app_name.clone())
                    .or_default()
                    .insert(filename, content);
            }
        }
    }

    if !found_old_layout {
        println!("No old machines/ directories found. Nothing to migrate.");
        return Ok(());
    }

    println!("\nFound {} machine(s) in old layout:", machine_data.len());
    for (machine_id, apps) in &machine_data {
        let file_count: usize = apps.values().map(|f| f.len()).sum();
        println!(
            "  {} — {} app(s), {} file(s)",
            machine_id,
            apps.len(),
            file_count
        );
    }

    if !confirm_operation("\nMigrate to branch-per-machine layout?", true)? {
        println!("Cancelled.");
        return Ok(());
    }

    // For each machine, create a branch and write files in flat layout
    for (machine_id, apps) in &machine_data {
        let branch_name = format!("machines/{}", machine_id);
        println!("\nCreating branch '{}'...", branch_name);

        // Start from main
        checkout_branch(repo_path, "main")?;
        create_branch(repo_path, &branch_name)?;

        // Write files in flat layout: apps/<app>/<filename>
        for (app_name, files) in apps {
            let app_dir = repo_path.join("apps").join(app_name);
            fs::create_dir_all(&app_dir)?;

            for (filename, content) in files {
                let dest = app_dir.join(filename);
                fs::write(&dest, content)?;
                log::debug!("Wrote {:?} on branch {}", dest, branch_name);
            }

            // Remove the machines/ subdirectory from this branch
            let machines_subdir = app_dir.join("machines");
            if machines_subdir.exists() {
                fs::remove_dir_all(&machines_subdir)?;
            }
        }

        // Commit and push the branch
        let msg = format!("Migrate {} to branch-per-machine layout", machine_id);
        commit_and_push(repo_path, &msg)?;
        println!("  ✓ Branch '{}' created and pushed", branch_name);

        // Update registry with branch info
        if let Some(info) = registry.machines.get_mut(machine_id) {
            info.branch = Some(branch_name.clone());
        }
    }

    // Clean up old machines/ directories from main
    println!("\nCleaning up old layout on main...");
    checkout_branch(repo_path, "main")?;

    for app_entry in fs::read_dir(&apps_dir)? {
        let app_dir = app_entry?.path();
        if !app_dir.is_dir() {
            continue;
        }
        let machines_dir = app_dir.join("machines");
        if machines_dir.exists() {
            fs::remove_dir_all(&machines_dir)?;
            log::debug!("Removed {:?} from main", machines_dir);
        }
    }

    // Save updated registry
    registry.save(repo_path)?;

    // Commit and push cleanup on main
    commit_and_push(repo_path, "Migrate to branch-per-machine layout (cleanup old directories)")?;
    println!("✓ Old machines/ directories removed from main");

    println!("\n✓ Migration complete!");
    println!("  Each machine now has its own branch: machines/<machine_id>");
    println!("  Use 'drifters push-app' to push to your machine's branch");
    println!("  Use 'drifters merge-app' to merge your branch into main");
    println!("  Use 'drifters pull-app' to pull from main");

    Ok(())
}
