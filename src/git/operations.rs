use crate::error::{DriftersError, Result};
use std::path::PathBuf;
use std::process::Command;

/// Run a git command inside `cwd`.  Returns trimmed stdout on success or a
/// `DriftersError::Git` carrying the trimmed stderr on failure.
fn git_run(cwd: &PathBuf, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(DriftersError::Git(stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn clone_repo(url: &str, path: &PathBuf) -> Result<()> {
    log::info!("Cloning repo {} to {:?}", url, path);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let output = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(DriftersError::Git(format!(
            "Failed to clone repository\nRepository URL: {}\nError: {}",
            url, stderr
        )));
    }

    log::info!("Successfully cloned repository");
    Ok(())
}

pub fn init_repo(path: &PathBuf) -> Result<()> {
    log::info!("Initializing new repository at {:?}", path);
    std::fs::create_dir_all(path)?;
    git_run(path, &["init"])?;
    log::info!("Successfully initialized repository");
    Ok(())
}

pub fn set_remote_origin(repo_path: &PathBuf, url: &str) -> Result<()> {
    git_run(repo_path, &["remote", "add", "origin", url])?;
    Ok(())
}

pub fn commit_and_push(repo_path: &PathBuf, message: &str) -> Result<()> {
    log::info!("Committing and pushing: {}", message);

    // Stage all changes (tracked + new files)
    git_run(repo_path, &["add", "."])?;

    // Guard: nothing-to-commit check.
    // `git diff --cached --quiet` exits 0 when the index is clean (nothing staged).
    let staged = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["diff", "--cached", "--quiet"])
        .status()?;

    if staged.success() {
        log::debug!("Nothing to commit (index clean), skipping push");
        return Ok(());
    }

    // Read author from git config; fall back to sensible defaults so drifters
    // works even on machines with no global git user config.
    let name = git_run(repo_path, &["config", "user.name"])
        .unwrap_or_else(|_| "Drifters User".to_string());
    let email = git_run(repo_path, &["config", "user.email"])
        .unwrap_or_else(|_| "drifters@localhost".to_string());

    git_run(
        repo_path,
        &[
            "-c", &format!("user.name={}", name),
            "-c", &format!("user.email={}", email),
            "commit", "-m", message,
        ],
    )?;

    log::debug!("Created commit: {}", message);

    push_to_remote(repo_path)
}

pub fn pull_latest(repo_path: &PathBuf) -> Result<()> {
    log::info!("Pulling latest from {:?}", repo_path);

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["pull", "--rebase"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(DriftersError::Git(format!(
            "Failed to pull latest changes\nError: {}",
            stderr
        )));
    }

    log::info!("Successfully pulled latest changes");
    Ok(())
}

fn push_to_remote(repo_path: &PathBuf) -> Result<()> {
    let branch = git_run(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|_| "main".to_string());

    let remote_url = git_run(repo_path, &["remote", "get-url", "origin"])
        .unwrap_or_else(|_| "unknown".to_string());

    log::debug!("Pushing {} to origin", branch);

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["push", "-u", "origin", &branch])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(DriftersError::Git(format!(
            "Failed to push to remote\nRepository URL: {}\nError: {}",
            remote_url, stderr
        )));
    }

    log::info!("Successfully pushed to remote");
    Ok(())
}
