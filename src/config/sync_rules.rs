use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRules {
    pub apps: HashMap<String, AppConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub files: Vec<PathBuf>,
    #[serde(default)]
    pub sync_mode: SyncMode,
    #[serde(default)]
    pub exceptions: HashMap<String, Vec<String>>,
    /// Optional selectors per file (for JSONPath, Regex, Lines modes)
    /// Key is filename, value is the selector string
    #[serde(default)]
    pub selectors: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    /// Sync the entire file
    #[default]
    Full,
    /// Use comment markers like #-start-sync- (for files that support comments)
    Markers,
    /// Select specific keys using JSONPath/YAMLPath (e.g., ".theme", ".keybindings[*]")
    #[serde(rename = "jsonpath")]
    JsonPath,
    /// Select specific line ranges (e.g., "10-50,100-150")
    Lines,
    /// Select content matching regex patterns
    Regex,
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

    pub fn get_app(&self, app_name: &str) -> Option<&AppConfig> {
        self.apps.get(app_name)
    }
}

impl Default for SyncRules {
    fn default() -> Self {
        Self::new()
    }
}
