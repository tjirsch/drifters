use crate::config::{LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, EphemeralRepoGuard};
use std::fs;
use std::path::PathBuf;

pub fn import_app(app_name: String, file_path: Option<PathBuf>) -> Result<()> {
    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Determine file path: use provided or default to <app>.toml in config repo
    let actual_file_path = match file_path {
        Some(path) => path,
        None => {
            let apps_dir = repo_path.join(".drifters").join("apps");
            fs::create_dir_all(&apps_dir)?;
            apps_dir.join(format!("{}.toml", app_name))
        }
    };

    log::info!("Importing app '{}' from {:?}", app_name, actual_file_path);

    // Load the app definition from file
    let file_content = fs::read_to_string(&actual_file_path)?;
    let file_rules: SyncRules = toml::from_str(&file_content)?;

    // Get the app config from the file
    let app_config = file_rules
        .apps
        .get(&app_name)
        .ok_or_else(|| {
            DriftersError::Config(format!(
                "App '{}' not found in file {:?}",
                app_name, actual_file_path
            ))
        })?
        .clone();

    // Load current sync rules
    let mut rules = SyncRules::load(repo_path)?;

    // Check if app already exists
    let is_update = rules.apps.contains_key(&app_name);

    // Update or add the app
    rules.apps.insert(app_name.clone(), app_config);

    // Save rules
    rules.save(repo_path)?;

    let action = if is_update { "Updated" } else { "Added" };
    println!("\n✓ {} '{}' from {:?}", action, app_name, actual_file_path);

    // Commit and push
    println!("\nCommitting changes...");
    let message = format!("{} {} app from file", action, app_name);
    commit_and_push(repo_path, &message)?;

    println!("✓ Changes committed and pushed");
    println!(
        "\nRun 'drifters merge --app {}' to apply the new rules",
        app_name
    );

    Ok(())
}

pub fn import_rules(file_path: Option<PathBuf>) -> Result<()> {
    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Determine file path: use provided or default to sync-rules.toml in config repo
    let actual_file_path = match file_path {
        Some(path) => path,
        None => repo_path.join(".drifters").join("sync-rules.toml"),
    };

    log::info!("Importing rules from {:?}", actual_file_path);

    // Load the rules from file
    let file_content = fs::read_to_string(&actual_file_path)?;
    let new_rules: SyncRules = toml::from_str(&file_content)?;

    // Save new rules (overwrites existing)
    new_rules.save(repo_path)?;

    println!("\n✓ Imported rules from {:?}", actual_file_path);
    println!("  {} app(s) imported", new_rules.apps.len());

    // Commit and push
    println!("\nCommitting changes...");
    let message = "Import sync rules from file";
    commit_and_push(repo_path, message)?;

    println!("✓ Changes committed and pushed");
    println!("\nRun 'drifters merge' to apply the new rules");

    Ok(())
}
