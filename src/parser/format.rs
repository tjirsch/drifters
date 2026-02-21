use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum FileFormat {
    Json,
    Yaml,
    Toml,
    Shell,
    Text,
}

pub fn detect_format(path: &Path) -> FileFormat {
    // Simple detection based on extension
    match path.extension().and_then(|s| s.to_str()) {
        Some("json") | Some("jsonc") => FileFormat::Json,
        Some("yaml") | Some("yml") => FileFormat::Yaml,
        Some("toml") => FileFormat::Toml,
        Some("sh") | Some("bash") | Some("zsh") => FileFormat::Shell,
        _ => FileFormat::Text,
    }
}
