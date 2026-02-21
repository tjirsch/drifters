use crate::config::{LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::EphemeralRepoGuard;
use std::fs;
use std::path::PathBuf;

pub fn export_app(app_name: String, file_path: PathBuf) -> Result<()> {
    log::info!("Exporting app '{}' to {:?}", app_name, file_path);

    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

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
    fs::write(&file_path, toml_content)?;

    println!("\n✓ Exported '{}' to {:?}", app_name, file_path);
    println!("\nYou can now:");
    println!("  - Edit the file");
    println!("  - Import it back: drifters import app {} --file {:?}", app_name, file_path);
    println!("  - Share it with others");

    Ok(())
}

pub fn export_rules(file_path: PathBuf) -> Result<()> {
    log::info!("Exporting rules to {:?}", file_path);

    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Load sync rules
    let rules = SyncRules::load(repo_path)?;

    // Serialize to TOML
    let toml_content = toml::to_string_pretty(&rules)?;

    // Write to file
    fs::write(&file_path, toml_content)?;

    println!("\n✓ Exported rules to {:?}", file_path);
    println!("  {} app(s) exported", rules.apps.len());

    println!("\nYou can now:");
    println!("  - Edit the file");
    println!("  - Import it back: drifters import rules --file {:?}", file_path);
    println!("  - Share your complete config");

    Ok(())
}
