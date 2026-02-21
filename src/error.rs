use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DriftersError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Merge conflict in file: {0}")]
    MergeConflict(PathBuf),

    #[error("Machine not registered: {0}")]
    MachineNotRegistered(String),

    #[error("App not found: {0}")]
    AppNotFound(String),

    #[error("Repository not initialized. Run 'drifters init <repo-url>' first")]
    RepoNotInitialized,

    #[error("Empty file detected: {0}. This might overwrite existing configs.")]
    EmptyFile(PathBuf),

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("Invalid sync mode: {0}")]
    InvalidSyncMode(String),
}

pub type Result<T> = std::result::Result<T, DriftersError>;
