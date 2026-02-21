use crate::config::LocalConfig;
use crate::error::Result;
use crate::git::EphemeralRepoGuard;
use std::process::Command;

pub fn show_history_rules(limit: usize) -> Result<()> {
    log::info!("Showing history of sync rules");

    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    println!("\nSync Rules History");
    println!("{}", "=".repeat(60));

    // Use git log to show history
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("log")
        .arg("--oneline")
        .arg("--decorate")
        .arg(format!("-{}", limit))
        .arg("--")
        .arg(".drifters/sync-rules.toml")
        .output()?;

    if output.status.success() {
        let log_output = String::from_utf8_lossy(&output.stdout);
        if log_output.trim().is_empty() {
            println!("No history found for sync-rules.toml");
        } else {
            println!("{}", log_output);
        }
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to get git log");
        eprintln!("Repository: {:?}", repo_path);
        eprintln!("Error: {}", err);
        return Err(crate::error::DriftersError::Config(
            "Unable to retrieve git history".to_string()
        ));
    }

    println!("\nTo see details:");
    println!("  drifters history rules --commit <hash>");
    println!("\nTo restore a version:");
    println!("  drifters restore rules --commit <hash>");

    Ok(())
}

pub fn show_history_app(app_name: String, limit: usize) -> Result<()> {
    log::info!("Showing history of app '{}'", app_name);

    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    println!("\nHistory for App: {}", app_name);
    println!("{}", "=".repeat(60));

    // Use git log with grep to find commits affecting this app
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("log")
        .arg("--oneline")
        .arg("--decorate")
        .arg(format!("-{}", limit))
        .arg("--grep")
        .arg(&app_name)
        .arg("--")
        .arg(".drifters/sync-rules.toml")
        .output()?;

    if output.status.success() {
        let log_output = String::from_utf8_lossy(&output.stdout);
        if log_output.trim().is_empty() {
            println!("No history found for app '{}'", app_name);
            println!("\nShowing all sync-rules.toml commits instead:");
            show_history_rules(limit)?;
        } else {
            println!("{}", log_output);
        }
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to get git log");
        eprintln!("Repository: {:?}", repo_path);
        eprintln!("Error: {}", err);
        return Err(crate::error::DriftersError::Config(
            "Unable to retrieve git history".to_string()
        ));
    }

    println!("\nTo see details:");
    println!("  drifters history app {} --commit <hash>", app_name);
    println!("\nTo restore a version:");
    println!("  drifters restore app {} --commit <hash>", app_name);

    Ok(())
}

pub fn show_commit_diff(commit: String, app_name: Option<String>) -> Result<()> {
    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    let title = if let Some(app) = &app_name {
        format!("Changes in commit {} (app: {})", commit, app)
    } else {
        format!("Changes in commit {}", commit)
    };

    println!("\n{}", title);
    println!("{}", "=".repeat(60));

    // Show the commit diff
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("show")
        .arg(&commit)
        .arg("--")
        .arg(".drifters/sync-rules.toml")
        .output()?;

    if output.status.success() {
        let diff_output = String::from_utf8_lossy(&output.stdout);
        println!("{}", diff_output);
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to show commit");
        eprintln!("Repository: {:?}", repo_path);
        eprintln!("Commit: {}", commit);
        eprintln!("Error: {}", err);
        return Err(crate::error::DriftersError::Config(
            "Unable to display commit diff".to_string()
        ));
    }

    Ok(())
}
