use crate::config::{LocalConfig, SyncRules};
use crate::error::Result;
use crate::git::EphemeralRepoGuard;

pub fn list_apps() -> Result<()> {
    log::info!("Listing apps");

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
        return Ok(());
    }

    println!("\nConfigured apps:");
    println!("{}", "=".repeat(60));

    for (app_name, app_config) in &rules.apps {
        println!("\n{}", app_name);
        println!("  Sync mode: {:?}", app_config.sync_mode);
        println!("  Files:");
        for file in &app_config.files {
            println!("    - {}", file.display());
        }

        // Show exceptions for this machine
        if let Some(exceptions) = app_config.exceptions.get(&config.machine_id) {
            if !exceptions.is_empty() {
                println!("  Excluded on this machine ({}):", config.machine_id);
                for exc in exceptions {
                    println!("    - {}", exc);
                }
            }
        }

        // Show selectors if any
        if !app_config.selectors.is_empty() {
            println!("  Selectors:");
            for (file, selector) in &app_config.selectors {
                println!("    {}: {}", file, selector);
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Total apps: {}", rules.apps.len());

    Ok(())
}
