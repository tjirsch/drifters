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
    Git(String),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("App not found: {0}")]
    AppNotFound(String),

    #[error("Repository not initialized. Run 'drifters init <repo-url>' first")]
    RepoNotInitialized,

    #[error("User cancelled operation")]
    UserCancelled,
}

pub type Result<T> = std::result::Result<T, DriftersError>;
