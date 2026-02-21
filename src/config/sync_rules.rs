use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRules {
    pub apps: HashMap<String, AppConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Base include patterns (glob patterns supported)
    #[serde(default)]
    pub include: Vec<String>,

    /// Base exclude patterns (glob patterns supported)
    #[serde(default)]
    pub exclude: Vec<String>,

    /// macOS-specific include patterns
    #[serde(rename = "include-macos", default)]
    pub include_macos: Vec<String>,

    /// Linux-specific include patterns
    #[serde(rename = "include-linux", default)]
    pub include_linux: Vec<String>,

    /// Windows-specific include patterns
    #[serde(rename = "include-windows", default)]
    pub include_windows: Vec<String>,

    /// macOS-specific exclude patterns
    #[serde(rename = "exclude-macos", default)]
    pub exclude_macos: Vec<String>,

    /// Linux-specific exclude patterns
    #[serde(rename = "exclude-linux", default)]
    pub exclude_linux: Vec<String>,

    /// Windows-specific exclude patterns
    #[serde(rename = "exclude-windows", default)]
    pub exclude_windows: Vec<String>,

    /// Machine-specific overrides
    #[serde(default)]
    pub machines: HashMap<String, MachineOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MachineOverride {
    #[serde(default)]
    pub include: Vec<String>,

    #[serde(default)]
    pub exclude: Vec<String>,
}

impl SyncRules {
    pub fn new() -> Self {
        Self {
            apps: HashMap::new(),
        }
    }

    pub fn load(repo_path: &PathBuf) -> Result<Self> {
        let rules_path = repo_path.join(".drifters").join("sync-rules.toml");

        if !rules_path.exists() {
            return Ok(Self::new());
        }

        let contents = std::fs::read_to_string(&rules_path)?;
        let rules: SyncRules = toml::from_str(&contents)?;
        Ok(rules)
    }

    pub fn save(&self, repo_path: &PathBuf) -> Result<()> {
        let drifters_dir = repo_path.join(".drifters");
        std::fs::create_dir_all(&drifters_dir)?;

        let rules_path = drifters_dir.join("sync-rules.toml");
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&rules_path, contents)?;
        Ok(())
    }

    pub fn add_app(&mut self, app_name: String, config: AppConfig) {
        self.apps.insert(app_name, config);
    }
}

impl Default for SyncRules {
    fn default() -> Self {
        Self::new()
    }
}
