use crate::error::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Read all files for an app on the current branch.
/// Returns a map of filename → content.
pub fn read_app_files(repo_path: &Path, app_name: &str) -> Result<HashMap<String, String>> {
    let app_dir = repo_path.join("apps").join(app_name);
    let mut files = HashMap::new();

    if !app_dir.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(&app_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            let content = fs::read_to_string(&path)?;
            files.insert(filename.to_string(), content);
        }
    }

    Ok(files)
}

