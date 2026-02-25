use crate::git::get_file_commit_time;
use crate::error::Result;
use std::collections::HashMap;
use std::fs;

/// A single machine's version of a config file, together with the git commit
/// timestamp of the most recent push from that machine.
///
/// `committed_at` is `None` for files that have no git history (e.g. a repo
/// that predates timestamp tracking).  The merge logic treats `None` as epoch 0
/// — always older than any real push.
pub struct MachineVersion {
    pub content: String,
    pub committed_at: Option<u64>,
}

/// Collect all machine versions of a specific file from the repo's machines directory.
///
/// Reads `machines_dir/<machine-id>/<filename>` for every subdirectory and
/// queries `git log` to obtain the commit timestamp for each file so that the
/// merge layer can apply last-write-wins semantics.
///
/// If `filter_machine` is `Some(id)`, only that machine's version is returned.
pub fn collect_machine_versions(
    repo_path: &std::path::Path,
    machines_dir: &std::path::Path,
    filename: &str,
    filter_machine: Option<&str>,
) -> Result<HashMap<String, MachineVersion>> {
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

            // Compute relative path for git log (e.g. "apps/zsh/machines/laptop/.zshrc")
            let relative_path = file_path
                .strip_prefix(repo_path)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();

            let committed_at = get_file_commit_time(repo_path, &relative_path);

            versions.insert(machine_id, MachineVersion { content, committed_at });
        }
    }

    Ok(versions)
}
