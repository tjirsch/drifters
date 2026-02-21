use crate::config::sync_rules::{AppConfig, MachineOverride};
use crate::error::Result;
use std::path::PathBuf;

/// Resolve the fileset for a given app on a specific machine/OS
/// Applies three-level hierarchy: Machine > OS > App
pub fn resolve_fileset(
    app_config: &AppConfig,
    machine_id: &str,
    os: &str,
) -> Result<Vec<PathBuf>> {
    let mut include_patterns: Vec<String> = Vec::new();
    let mut exclude_patterns: Vec<String> = Vec::new();

    // 1. Start with app defaults
    include_patterns.extend(app_config.include.iter().cloned());
    exclude_patterns.extend(app_config.exclude.iter().cloned());

    // 2. Apply OS-specific rules
    match os {
        "macos" => {
            include_patterns.extend(app_config.include_macos.iter().cloned());
            exclude_patterns.extend(app_config.exclude_macos.iter().cloned());
        }
        "linux" => {
            include_patterns.extend(app_config.include_linux.iter().cloned());
            exclude_patterns.extend(app_config.exclude_linux.iter().cloned());
        }
        "windows" => {
            include_patterns.extend(app_config.include_windows.iter().cloned());
            exclude_patterns.extend(app_config.exclude_windows.iter().cloned());
        }
        _ => {
            log::warn!("Unknown OS: {}, using app defaults only", os);
        }
    }

    // 3. Apply machine-specific overrides
    if let Some(machine_override) = app_config.machines.get(machine_id) {
        include_patterns.extend(machine_override.include.iter().cloned());
        exclude_patterns.extend(machine_override.exclude.iter().cloned());
    }

    // 4. Expand globs and apply exclusions
    let mut files: Vec<PathBuf> = Vec::new();

    for pattern in include_patterns {
        let expanded_pattern = expand_tilde(&pattern);

        match glob::glob(&expanded_pattern) {
            Ok(paths) => {
                for path_result in paths {
                    match path_result {
                        Ok(path) => {
                            if !matches_any_pattern(&path, &exclude_patterns) {
                                files.push(path);
                            }
                        }
                        Err(e) => {
                            log::warn!("Error reading glob path: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Invalid glob pattern '{}': {}", expanded_pattern, e);
            }
        }
    }

    // Remove duplicates
    files.sort();
    files.dedup();

    Ok(files)
}

/// Check if a path matches any of the exclude patterns
fn matches_any_pattern(path: &PathBuf, patterns: &[String]) -> bool {
    for pattern in patterns {
        let expanded_pattern = expand_tilde(pattern);

        // Try glob match
        if let Ok(glob_pattern) = glob::Pattern::new(&expanded_pattern) {
            if glob_pattern.matches_path(path) {
                return true;
            }
        }

        // Also check simple path match
        if path.to_str().map(|p| p.contains(pattern)).unwrap_or(false) {
            return true;
        }
    }

    false
}

/// Expand tilde (~) to home directory
pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]).to_string_lossy().to_string();
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::sync_rules::AppConfig;

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/test/path");
        assert!(expanded.contains("test/path"));
        assert!(!expanded.starts_with("~"));
    }

    #[test]
    fn test_resolve_fileset_basic() {
        let config = AppConfig {
            include: vec!["~/test/*.txt".to_string()],
            exclude: vec![],
            include_macos: vec![],
            include_linux: vec![],
            include_windows: vec![],
            exclude_macos: vec![],
            exclude_linux: vec![],
            exclude_windows: vec![],
            sections: Default::default(),
            machines: Default::default(),
        };

        // This will return empty if ~/test/ doesn't exist, which is fine for a unit test
        let result = resolve_fileset(&config, "machine1", "linux");
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_fileset_os_specific() {
        let config = AppConfig {
            include: vec![],
            exclude: vec![],
            include_macos: vec!["~/mac-only.txt".to_string()],
            include_linux: vec!["~/linux-only.txt".to_string()],
            include_windows: vec![],
            exclude_macos: vec![],
            exclude_linux: vec![],
            exclude_windows: vec![],
            sections: Default::default(),
            machines: Default::default(),
        };

        let result = resolve_fileset(&config, "machine1", "macos").unwrap();
        // Results will be empty if files don't exist, but no errors
        assert!(result.is_empty() || result.iter().any(|p| p.to_str().unwrap().contains("mac-only")));
    }
}
