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

    // Fetch first (always works even on empty repos)
    let fetch = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["fetch", "origin"])
        .output()?;

    if !fetch.status.success() {
        let stderr = String::from_utf8_lossy(&fetch.stderr).trim().to_string();
        // Empty repos may fail to fetch — not an error
        if !stderr.contains("no matching remote head") {
            return Err(DriftersError::Git(format!(
                "Failed to fetch from origin\nError: {}",
                stderr
            )));
        }
        log::debug!("Fetch found no remote head (empty repo?)");
        return Ok(());
    }

    // Check if HEAD exists (repo has at least one commit)
    let has_head = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["rev-parse", "HEAD"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_head {
        log::debug!("No HEAD commit, skipping pull (empty repo)");
        return Ok(());
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["pull", "--rebase"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        // "There is no tracking information" happens on branches with no upstream
        if stderr.contains("no tracking information") || stderr.contains("You are not currently on a branch") {
            log::debug!("No tracking branch, skipping pull");
            return Ok(());
        }
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

// ─── Branch operations ──────────────────────────────────────────────────────

/// Create and checkout a new branch from the current HEAD.
pub fn create_branch(repo_path: &PathBuf, branch_name: &str) -> Result<()> {
    git_run(repo_path, &["checkout", "-b", branch_name])?;
    Ok(())
}

/// Checkout an existing branch.
pub fn checkout_branch(repo_path: &PathBuf, branch_name: &str) -> Result<()> {
    git_run(repo_path, &["checkout", branch_name])?;
    Ok(())
}

/// Fetch a specific branch from origin.
pub fn fetch_branch(repo_path: &PathBuf, branch_name: &str) -> Result<()> {
    git_run(repo_path, &["fetch", "origin", branch_name])?;
    Ok(())
}

/// Merge a source branch into the current branch.
/// Returns Ok(()) on clean merge, Err(MergeConflict) if conflicts arise.
pub fn merge_branch(repo_path: &PathBuf, source_branch: &str) -> Result<()> {
    let name = git_run(repo_path, &["config", "user.name"])
        .unwrap_or_else(|_| "Drifters User".to_string());
    let email = git_run(repo_path, &["config", "user.email"])
        .unwrap_or_else(|_| "drifters@localhost".to_string());

    let result = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("-c")
        .arg(format!("user.name={}", name))
        .arg("-c")
        .arg(format!("user.email={}", email))
        .args(["merge", source_branch])
        .output()?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        if stderr.contains("CONFLICT") || stderr.contains("Automatic merge failed") {
            return Err(DriftersError::MergeConflict(stderr));
        }
        return Err(DriftersError::Git(stderr));
    }
    Ok(())
}

/// Run git mergetool to resolve conflicts interactively.
pub fn run_mergetool(repo_path: &PathBuf) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("mergetool")
        .status()?;

    if !status.success() {
        return Err(DriftersError::Git(
            "git mergetool failed or was cancelled".to_string(),
        ));
    }
    Ok(())
}

/// List all branches (local and remote).
pub fn list_branches(repo_path: &PathBuf) -> Result<Vec<String>> {
    let output = git_run(repo_path, &["branch", "-a", "--format=%(refname:short)"])?;
    Ok(output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// Checkout a branch, creating it from a base branch if it doesn't exist locally.
/// If the branch exists on the remote but not locally, creates a tracking branch.
pub fn checkout_or_create_branch(
    repo_path: &PathBuf,
    branch_name: &str,
    base_branch: &str,
) -> Result<()> {
    // Try checkout first (works if branch exists locally)
    if checkout_branch(repo_path, branch_name).is_ok() {
        return Ok(());
    }

    // Try to fetch and track from remote
    if fetch_branch(repo_path, branch_name).is_ok() {
        let remote_ref = format!("origin/{}", branch_name);
        if git_run(repo_path, &["checkout", "-b", branch_name, "--track", &remote_ref]).is_ok() {
            return Ok(());
        }
    }

    // Branch doesn't exist anywhere — create from base
    checkout_branch(repo_path, base_branch)?;
    create_branch(repo_path, branch_name)?;
    Ok(())
}

/// Merge a branch without committing (for dry-run).
/// Returns Ok(true) if clean, Ok(false) if conflicts, and aborts the merge.
pub fn merge_dry_run(repo_path: &PathBuf, source_branch: &str) -> Result<(bool, String)> {
    let name = git_run(repo_path, &["config", "user.name"])
        .unwrap_or_else(|_| "Drifters User".to_string());
    let email = git_run(repo_path, &["config", "user.email"])
        .unwrap_or_else(|_| "drifters@localhost".to_string());

    let result = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("-c")
        .arg(format!("user.name={}", name))
        .arg("-c")
        .arg(format!("user.email={}", email))
        .args(["merge", "--no-commit", source_branch])
        .output()?;

    let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();

    // Get diff of what would change
    let diff = git_run(repo_path, &["diff", "--cached", "--stat"])
        .unwrap_or_default();

    // Abort the merge
    let _ = git_run(repo_path, &["merge", "--abort"]);

    if result.status.success() {
        Ok((true, diff))
    } else {
        Ok((false, format!("{}\n{}", stderr, diff)))
    }
}

/// Commit merge result (after mergetool resolution).
pub fn commit_merge(repo_path: &PathBuf, message: &str) -> Result<()> {
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
    Ok(())
}
