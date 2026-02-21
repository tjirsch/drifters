use crate::error::{DriftersError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub machine_id: String,
    pub repo_url: String,
    // Note: repo_path is ephemeral (derived from get_temp_repo_path())
    // It's cloned/pulled on each command and deleted after
    #[serde(skip)]
    pub repo_path: PathBuf,
}

impl LocalConfig {
    pub fn new(machine_id: String, repo_url: String) -> Self {
        Self {
            machine_id,
            repo_url,
            repo_path: Self::get_temp_repo_path().unwrap_or_default(),
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        if !config_path.exists() {
            return Err(DriftersError::RepoNotInitialized);
        }

        let contents = std::fs::read_to_string(&config_path)?;
        let mut config: LocalConfig = toml::from_str(&contents)?;

        // Set ephemeral repo path
        config.repo_path = Self::get_temp_repo_path()?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;
        Ok(())
    }

    pub fn config_file_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| DriftersError::Config("Could not find home directory".to_string()))?;
        Ok(home.join(".config").join("drifters").join("config.toml"))
    }

    pub fn get_temp_repo_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| DriftersError::Config("Could not find home directory".to_string()))?;
        Ok(home.join(".config").join("drifters").join("tmp-repo"))
    }

    pub fn detect_machine_id() -> String {
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string())
    }
}
