use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::Result;
use crate::git::EphemeralRepoGuard;

pub fn list_apps(filter_app: Option<String>) -> Result<()> {
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

    // If filter_app is specified, only show that app
    let apps_to_show: Vec<(&String, &crate::config::AppConfig)> = if let Some(ref filter) = filter_app {
        if let Some(app_config) = rules.apps.get(filter) {
            vec![(filter, app_config)]
        } else {
            println!("App '{}' not found in sync rules.", filter);
            return Ok(());
        }
    } else {
        rules.apps.iter().collect()
    };

    if filter_app.is_some() {
        println!("\nApp details:");
    } else {
        println!("\nConfigured apps:");
    }
    println!("{}", "=".repeat(60));

    for (app_name, app_config) in &apps_to_show {
        println!("\n{}", app_name);

        // Show include patterns
        if !app_config.include.is_empty() {
            println!("  Include patterns:");
            for pattern in &app_config.include {
                println!("    - {}", pattern);
            }
        }

        // Show OS-specific includes
        if !app_config.include_macos.is_empty() {
            println!("  Include (macOS only):");
            for pattern in &app_config.include_macos {
                println!("    - {}", pattern);
            }
        }
        if !app_config.include_linux.is_empty() {
            println!("  Include (Linux only):");
            for pattern in &app_config.include_linux {
                println!("    - {}", pattern);
            }
        }
        if !app_config.include_windows.is_empty() {
            println!("  Include (Windows only):");
            for pattern in &app_config.include_windows {
                println!("    - {}", pattern);
            }
        }

        // Show exclude patterns
        if !app_config.exclude.is_empty() {
            println!("  Exclude patterns:");
            for pattern in &app_config.exclude {
                println!("    - {}", pattern);
            }
        }

        // Show machine-specific overrides for this machine
        if let Some(machine_override) = app_config.machines.get(&config.machine_id) {
            if !machine_override.include.is_empty() {
                println!("  Include on this machine ({}):", config.machine_id);
                for pattern in &machine_override.include {
                    println!("    - {}", pattern);
                }
            }
            if !machine_override.exclude.is_empty() {
                println!("  Excluded on this machine ({}):", config.machine_id);
                for pattern in &machine_override.exclude {
                    println!("    - {}", pattern);
                }
            }
        }

        // Show resolved fileset for this machine
        let fileset = resolve_fileset(
            app_config,
            &config.machine_id,
            std::env::consts::OS,
        )?;

        if !fileset.is_empty() {
            println!("  Resolved files ({}):", fileset.len());
            for (i, file) in fileset.iter().enumerate() {
                if i < 5 {
                    println!("    - {}", file.display());
                } else if i == 5 {
                    println!("    ... and {} more", fileset.len() - 5);
                    break;
                }
            }
        } else {
            println!("  (no files match for this machine/OS)");
        }
    }

    println!("\n{}", "=".repeat(60));
    if filter_app.is_none() {
        println!("Total apps: {}", rules.apps.len());
    }

    Ok(())
}

pub fn list_apps_simple() -> Result<()> {
    log::info!("Listing apps (simple)");

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

    println!("\nConfigured apps ({}):", rules.apps.len());
    let mut app_names: Vec<_> = rules.apps.keys().collect();
    app_names.sort();

    for app_name in app_names {
        // Show resolved file count for this machine
        let app_config = rules.apps.get(app_name).unwrap();
        let fileset = resolve_fileset(
            app_config,
            &config.machine_id,
            std::env::consts::OS,
        )?;

        let file_count = fileset.len();
        if file_count > 0 {
            println!("  {} ({} file{})", app_name, file_count, if file_count == 1 { "" } else { "s" });
        } else {
            println!("  {} (no files on this machine/OS)", app_name);
        }
    }

    Ok(())
}

pub fn list_rules() -> Result<()> {
    log::info!("Listing rules");

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo
    println!("Fetching latest sync rules...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Read the raw sync-rules.toml file
    let rules_path = repo_path.join(".drifters").join("sync-rules.toml");

    if !rules_path.exists() {
        println!("No sync-rules.toml found.");
        return Ok(());
    }

    let rules_content = std::fs::read_to_string(&rules_path)?;

    println!("\n{}", "=".repeat(60));
    println!("Current sync-rules.toml:");
    println!("{}", "=".repeat(60));
    println!("{}", rules_content);
    println!("{}", "=".repeat(60));

    Ok(())
}
