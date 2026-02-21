use crate::config::LocalConfig;
use crate::error::Result;
use crate::git::{clone_repo, pull_latest};
use std::path::PathBuf;

/// Set up ephemeral repo for this command
/// Clones if doesn't exist, pulls if it does
pub fn setup_ephemeral_repo(config: &LocalConfig) -> Result<PathBuf> {
    let temp_repo = LocalConfig::get_temp_repo_path()?;

    if temp_repo.exists() {
        log::debug!("Temp repo exists, pulling latest");
        pull_latest(&temp_repo)?;
    } else {
        log::debug!("Cloning repo to temp location");
        clone_repo(&config.repo_url, &temp_repo)?;
    }

    Ok(temp_repo)
}

/// Clean up ephemeral repo after command completes
pub fn cleanup_ephemeral_repo() -> Result<()> {
    let temp_repo = LocalConfig::get_temp_repo_path()?;

    if temp_repo.exists() {
        log::debug!("Cleaning up temp repo at {:?}", temp_repo);
        std::fs::remove_dir_all(&temp_repo)?;
    }

    Ok(())
}

/// RAII guard that ensures cleanup happens even on early return or panic
pub struct EphemeralRepoGuard {
    repo_path: PathBuf,
}

impl EphemeralRepoGuard {
    pub fn new(config: &LocalConfig) -> Result<Self> {
        let repo_path = setup_ephemeral_repo(config)?;
        Ok(Self { repo_path })
    }

    pub fn path(&self) -> &PathBuf {
        &self.repo_path
    }
}

impl Drop for EphemeralRepoGuard {
    fn drop(&mut self) {
        if let Err(e) = cleanup_ephemeral_repo() {
            log::warn!("Failed to cleanup ephemeral repo: {}", e);
        }
    }
}
