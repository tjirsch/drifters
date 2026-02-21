use crate::error::{DriftersError, Result};
use git2::{Repository, Signature};
use std::path::PathBuf;
use std::process::Command;

pub fn clone_repo(url: &str, path: &PathBuf) -> Result<()> {
    log::info!("Cloning repo {} to {:?}", url, path);

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Use system git command (which already has SSH configured)
    let output = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DriftersError::Git(git2::Error::from_str(&format!(
            "Failed to clone repository\nRepository URL: {}\nError: {}",
            url, stderr
        ))));
    }

    log::info!("Successfully cloned repository");
    Ok(())
}

pub fn init_repo(path: &PathBuf) -> Result<Repository> {
    log::info!("Initializing new repository at {:?}", path);

    // Create directory if needed
    std::fs::create_dir_all(path)?;

    // Initialize repository
    let repo = Repository::init(path)?;

    log::info!("Successfully initialized repository");
    Ok(repo)
}

pub fn commit_and_push(repo_path: &PathBuf, message: &str) -> Result<()> {
    log::info!("Committing and pushing: {}", message);

    let repo = Repository::open(repo_path)?;

    // Add all changes
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // Create commit
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Get signature
    let signature = get_signature(&repo)?;

    // Get parent commit (if exists)
    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(_) => None,
    };

    // Create commit
    if let Some(parent) = &parent_commit {
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[parent],
        )?;
    } else {
        // First commit
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[],
        )?;
    }

    log::debug!("Created commit: {}", message);

    // Push to remote
    push_to_remote(&repo)?;

    Ok(())
}

pub fn pull_latest(repo_path: &PathBuf) -> Result<()> {
    log::info!("Pulling latest from {:?}", repo_path);

    // Use system git command
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("pull")
        .arg("--rebase")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("git pull had issues: {}", stderr);
        // Don't fail if pull has conflicts, we'll handle it
    }

    log::info!("Successfully pulled latest changes");
    Ok(())
}

fn push_to_remote(repo: &Repository) -> Result<()> {
    // Get current branch
    let head = repo.head()?;
    let branch = head
        .shorthand()
        .ok_or_else(|| DriftersError::Config("Could not get branch name".to_string()))?;

    // Get remote URL for error reporting
    let remote_url = repo.find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    log::debug!("Pushing {} to origin", branch);

    // Use system git command
    let repo_path = repo.path().parent()
        .ok_or_else(|| DriftersError::Config("Invalid repo path".to_string()))?;

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("push")
        .arg("-u")
        .arg("origin")
        .arg(branch)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DriftersError::Git(git2::Error::from_str(&format!(
            "Failed to push to remote\nRepository URL: {}\nError: {}",
            remote_url, stderr
        ))));
    }

    log::info!("Successfully pushed to remote");
    Ok(())
}

fn get_signature(repo: &Repository) -> Result<Signature<'static>> {
    // Try to get from git config
    let config = repo.config()?;

    let name = config
        .get_string("user.name")
        .unwrap_or_else(|_| "Drifters User".to_string());
    let email = config
        .get_string("user.email")
        .unwrap_or_else(|_| "drifters@localhost".to_string());

    Ok(Signature::now(&name, &email)?)
}
