use crate::config::{LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, EphemeralRepoGuard};
use std::fs;
use std::process::Command;

pub fn restore_app(app_name: String, commit: String) -> Result<()> {
    log::info!("Restoring app '{}' from commit {}", app_name, commit);

    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Get the old version of sync-rules.toml
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("show")
        .arg(format!("{}:.drifters/sync-rules.toml", commit))
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(DriftersError::Config(format!(
            "Failed to get file from commit {}: {}",
            commit, err
        )));
    }

    let old_content = String::from_utf8_lossy(&output.stdout);
    let old_rules: SyncRules = toml::from_str(&old_content)?;

    // Get the app config from old version
    let old_app_config = old_rules
        .apps
        .get(&app_name)
        .ok_or_else(|| {
            DriftersError::Config(format!(
                "App '{}' not found in commit {}",
                app_name, commit
            ))
        })?
        .clone();

    // Load current rules
    let mut current_rules = SyncRules::load(repo_path)?;

    // Replace with old version
    current_rules.apps.insert(app_name.clone(), old_app_config);

    // Save
    current_rules.save(repo_path)?;

    println!(
        "\n✓ Restored '{}' from commit {}",
        app_name,
        &commit[..7.min(commit.len())]
    );

    // Commit and push
    println!("\nCommitting changes...");
    let message = format!("Restore {} app from commit {}", app_name, &commit[..7.min(commit.len())]);
    commit_and_push(repo_path, &message)?;

    println!("✓ Changes committed and pushed");
    println!(
        "\nRun 'drifters merge --app {}' to apply the restored rules",
        app_name
    );

    Ok(())
}

pub fn restore_rules(commit: String) -> Result<()> {
    log::info!("Restoring all rules from commit {}", commit);

    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Get the old version of sync-rules.toml
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("show")
        .arg(format!("{}:.drifters/sync-rules.toml", commit))
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(DriftersError::Config(format!(
            "Failed to get file from commit {}: {}",
            commit, err
        )));
    }

    let old_content = String::from_utf8_lossy(&output.stdout);
    let old_rules: SyncRules = toml::from_str(&old_content)?;

    // Write directly to file
    let rules_path = repo_path.join(".drifters").join("sync-rules.toml");
    fs::write(&rules_path, old_content.as_bytes())?;

    println!(
        "\n✓ Restored all rules from commit {}",
        &commit[..7.min(commit.len())]
    );
    println!("  {} app(s) restored", old_rules.apps.len());

    // Commit and push
    println!("\nCommitting changes...");
    let message = format!("Restore sync rules from commit {}", &commit[..7.min(commit.len())]);
    commit_and_push(repo_path, &message)?;

    println!("✓ Changes committed and pushed");
    println!("\nRun 'drifters merge' to apply the restored rules");

    Ok(())
}
