use crate::config::{LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, EphemeralRepoGuard};
use std::fs;
use std::path::PathBuf;

pub fn export_app(app_name: String, file_path: Option<PathBuf>) -> Result<()> {
    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Determine file path: use provided or default to <app>.toml in config repo
    let apps_dir = repo_path.join(".drifters").join("apps");
    fs::create_dir_all(&apps_dir)?;

    let is_default_location = file_path.is_none();
    let actual_file_path = match file_path {
        Some(path) => path,
        None => apps_dir.join(format!("{}.toml", app_name)),
    };

    log::info!("Exporting app '{}' to {:?}", app_name, actual_file_path);

    // Load sync rules
    let rules = SyncRules::load(repo_path)?;

    // Get the app config
    let app_config = rules
        .apps
        .get(&app_name)
        .ok_or_else(|| DriftersError::AppNotFound(app_name.clone()))?;

    // Create a new SyncRules with just this app
    let mut export_rules = SyncRules::new();
    export_rules.apps.insert(app_name.clone(), app_config.clone());

    // Serialize to TOML
    let toml_content = toml::to_string_pretty(&export_rules)?;

    // Write to file
    fs::write(&actual_file_path, toml_content)?;

    println!("\n✓ Exported '{}' to {:?}", app_name, actual_file_path);

    // If saving to config repo, commit and push
    if is_default_location {
        println!("\nCommitting to config repo...");
        let message = format!("Export {} app definition", app_name);
        commit_and_push(repo_path, &message)?;
        println!("✓ Committed and pushed to config repo");
    }

    println!("\nYou can now:");
    println!("  - Edit: {:?}", actual_file_path);
    println!("  - Import: drifters import-app {}", app_name);
    println!("  - Share with others");

    Ok(())
}

pub fn export_rules(file_path: Option<PathBuf>) -> Result<()> {
    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Determine file path: use provided or default to sync-rules.toml (which already exists)
    let is_default_location = file_path.is_none();
    let actual_file_path = match file_path {
        Some(path) => path,
        None => repo_path.join(".drifters").join("sync-rules.toml"),
    };

    log::info!("Exporting rules to {:?}", actual_file_path);

    // Load sync rules
    let rules = SyncRules::load(repo_path)?;

    // Serialize to TOML
    let toml_content = toml::to_string_pretty(&rules)?;

    // Write to file
    fs::write(&actual_file_path, toml_content)?;

    println!("\n✓ Exported rules to {:?}", actual_file_path);
    println!("  {} app(s) exported", rules.apps.len());

    // Note: sync-rules.toml in config repo is already the canonical source,
    // so we don't need to commit if that's where we exported to
    if is_default_location {
        println!("\n(This is the canonical sync-rules.toml - no commit needed)");
    }

    println!("\nYou can now:");
    println!("  - Edit the file");
    println!("  - Import: drifters import-rules");
    println!("  - Share your complete config");

    Ok(())
}
