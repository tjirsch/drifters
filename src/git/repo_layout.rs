use crate::error::Result;
use std::collections::HashMap;
use std::fs;

/// Collect all machine versions of a specific file from the repo's machines directory.
///
/// Reads `machines_dir/<machine-id>/<filename>` for every subdirectory.
/// If `filter_machine` is `Some(id)`, only that machine's version is returned.
pub fn collect_machine_versions(
    machines_dir: &std::path::Path,
    filename: &str,
    filter_machine: Option<&str>,
) -> Result<HashMap<String, String>> {
    let mut versions = HashMap::new();

    if !machines_dir.exists() {
        return Ok(versions);
    }

    for entry in fs::read_dir(machines_dir)? {
        let machine_dir = entry?.path();

        if !machine_dir.is_dir() {
            continue;
        }

        let machine_id = machine_dir
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                crate::error::DriftersError::Config(format!(
                    "Invalid machine directory name: {:?}",
                    machine_dir
                ))
            })?
            .to_string();

        // Apply optional filter
        if let Some(filter) = filter_machine {
            if machine_id != filter {
                continue;
            }
        }

        let file_path = machine_dir.join(filename);
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            versions.insert(machine_id, content);
        }
    }

    Ok(versions)
}
