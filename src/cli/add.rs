use crate::config::{AppConfig, LocalConfig, SyncRules};
use crate::error::Result;
use crate::git::{commit_and_push, EphemeralRepoGuard};
use std::io::{self, Write};

pub fn add_app(app_name: String) -> Result<()> {
    log::info!("Adding app: {}", app_name);

    // Load local config
    let config = LocalConfig::load()?;

    // Set up ephemeral repo
    println!("Setting up repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Load sync rules
    let mut rules = SyncRules::load(repo_path)?;

    // Check if app already exists
    if rules.apps.contains_key(&app_name) {
        println!("App '{}' is already configured", app_name);
        return Ok(());
    }

    println!("Adding app '{}'", app_name);
    println!("\nEnter file patterns to include (one per line, empty line to finish):");
    println!("Examples:");
    println!("  ~/.config/zed/settings.json");
    println!("  ~/.config/nvim/**/*.lua");
    println!("  ~/.zshrc");

    let mut include_patterns = Vec::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed = input.trim();

        if trimmed.is_empty() {
            break;
        }

        include_patterns.push(trimmed.to_string());
        println!("  Added: {}", trimmed);
    }

    if include_patterns.is_empty() {
        println!("No patterns specified, cancelling");
        return Ok(());
    }

    // Ask for optional exclude patterns
    println!("\nEnter file patterns to exclude (optional, empty line to skip):");
    println!("Examples:");
    println!("  ~/.config/zed/workspace-*.json");
    println!("  ~/.config/zed/cache/**");

    let mut exclude_patterns = Vec::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed = input.trim();

        if trimmed.is_empty() {
            break;
        }

        exclude_patterns.push(trimmed.to_string());
        println!("  Added exclusion: {}", trimmed);
    }

    println!("\nNote: Files will be scanned for section tags automatically.");
    println!("Use '# drifters::exclude::start' and '# drifters::exclude::stop' to exclude sections.");

    // Create app config
    let app_config = AppConfig {
        include: include_patterns,
        exclude: exclude_patterns,
        include_macos: vec![],
        include_linux: vec![],
        include_windows: vec![],
        exclude_macos: vec![],
        exclude_linux: vec![],
        exclude_windows: vec![],
        sections: Default::default(),
        machines: Default::default(),
    };

    // Add to rules
    rules.add_app(app_name.clone(), app_config);

    // Save rules
    rules.save(repo_path)?;
    println!("\n✓ Added '{}' to sync rules", app_name);

    // Commit and push
    println!("\nCommitting changes...");
    commit_and_push(
        repo_path,
        &format!("Add {} app from {}", app_name, config.machine_id),
    )?;

    println!("✓ Changes committed and pushed");
    println!("\nYou can now use:");
    println!("  drifters push {} - to push configs for this app", app_name);
    println!("  drifters pull {} - to pull configs for this app", app_name);

    Ok(())
}
