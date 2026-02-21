use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineRegistry {
    pub machines: HashMap<String, MachineInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    pub os: String,
    pub last_sync: Option<DateTime<Utc>>,
}

impl MachineRegistry {
    pub fn new() -> Self {
        Self {
            machines: HashMap::new(),
        }
    }

    pub fn load(repo_path: &PathBuf) -> Result<Self> {
        let machines_path = repo_path.join(".drifters").join("machines.toml");

        if !machines_path.exists() {
            return Ok(Self::new());
        }

        let contents = std::fs::read_to_string(&machines_path)?;
        let registry: MachineRegistry = toml::from_str(&contents)?;
        Ok(registry)
    }

    pub fn save(&self, repo_path: &PathBuf) -> Result<()> {
        let drifters_dir = repo_path.join(".drifters");
        std::fs::create_dir_all(&drifters_dir)?;

        let machines_path = drifters_dir.join("machines.toml");
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&machines_path, contents)?;
        Ok(())
    }

    pub fn register_machine(&mut self, machine_id: String, os: String) {
        self.machines.insert(
            machine_id,
            MachineInfo {
                os,
                last_sync: Some(Utc::now()),
            },
        );
    }

    pub fn detect_os() -> String {
        std::env::consts::OS.to_string()
    }
}

impl Default for MachineRegistry {
    fn default() -> Self {
        Self::new()
    }
}
