use crate::config::{LocalConfig, MachineOverride, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, EphemeralRepoGuard};

pub fn exclude_file(app_name: String, filename: String) -> Result<()> {
    log::info!("Excluding {} from {} on this machine", filename, app_name);

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo
    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Load sync rules
    let mut rules = SyncRules::load(repo_path)?;

    // Check if app exists
    let app_config = rules
        .apps
        .get_mut(&app_name)
        .ok_or_else(|| DriftersError::AppNotFound(app_name.clone()))?;

    // Add exclusion for this machine
    let machine_override = app_config
        .machines
        .entry(config.machine_id.clone())
        .or_insert_with(MachineOverride::default);

    // Add the filename as an exclude pattern
    let exclude_pattern = format!("**/{}", filename);

    if machine_override.exclude.contains(&exclude_pattern) {
        println!(
            "File '{}' is already excluded for {} on machine '{}'",
            filename, app_name, config.machine_id
        );
        return Ok(());
    }

    machine_override.exclude.push(exclude_pattern);

    // Save rules
    rules.save(repo_path)?;
    println!(
        "\n✓ Excluded '{}' from {} on machine '{}'",
        filename, app_name, config.machine_id
    );

    // Commit and push
    println!("\nCommitting changes...");
    commit_and_push(
        repo_path,
        &format!(
            "Exclude {} from {} on {}",
            filename, app_name, config.machine_id
        ),
    )?;

    println!("✓ Changes committed and pushed");
    println!(
        "\nThis file will no longer be synced to machine '{}'",
        config.machine_id
    );

    Ok(())
}
