use crate::config::{resolve_fileset, LocalConfig, SyncRules};
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

        // Show section overrides if any
        if !app_config.sections.is_empty() {
            println!("  Section processing:");
            for (file, enabled) in &app_config.sections {
                if *enabled {
                    println!("    {}: enabled", file);
                } else {
                    println!("    {}: disabled (full file sync)", file);
                }
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Total apps: {}", rules.apps.len());

    Ok(())
}
