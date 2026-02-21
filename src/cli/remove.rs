use crate::config::{LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, EphemeralRepoGuard};

pub fn remove_app(app_name: String) -> Result<()> {
    log::info!("Removing app: {}", app_name);

    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Load current sync rules
    let mut rules = SyncRules::load(repo_path)?;

    // Check if app exists
    if !rules.apps.contains_key(&app_name) {
        return Err(DriftersError::AppNotFound(app_name));
    }

    // Remove the app
    rules.apps.remove(&app_name);

    // Save rules
    rules.save(repo_path)?;

    println!("\n✓ Removed '{}' from sync rules", app_name);

    // Commit and push
    println!("\nCommitting changes...");
    let message = format!("Remove {} app from sync", app_name);
    commit_and_push(repo_path, &message)?;

    println!("✓ Changes committed and pushed");
    println!("\nThe app has been removed from sync rules across all machines.");
    println!("Note: Local files on this machine are NOT deleted.");

    Ok(())
}
