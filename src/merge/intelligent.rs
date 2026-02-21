use crate::config::AppConfig;
use crate::error::Result;
use std::collections::HashMap;

/// Intelligently merge content from multiple machines
/// Returns the merged content that should be applied locally
pub fn intelligent_merge(
    all_versions: &HashMap<String, String>,
    current_machine_id: &str,
    _filename: &str,
    _app_config: &AppConfig,
) -> Result<String> {
    // Handle simple cases first
    if all_versions.is_empty() {
        return Err(crate::error::DriftersError::Config(
            "No versions available to merge".to_string(),
        ));
    }

    // If only one version exists, use it
    if all_versions.len() == 1 {
        return Ok(all_versions.values().next().unwrap().clone());
    }

    // If all versions are identical, use any
    let first_content = all_versions.values().next().unwrap();
    if all_versions.values().all(|v| v == first_content) {
        return Ok(first_content.clone());
    }

    // For now, implement a simple strategy: use the most common version
    // TODO: Implement proper three-way merge with conflict detection
    let merged = find_consensus_version(all_versions, current_machine_id)?;

    Ok(merged)
}

/// Find the version that appears most frequently across machines
/// If there's a tie, prefer the current machine's version
fn find_consensus_version(
    all_versions: &HashMap<String, String>,
    current_machine_id: &str,
) -> Result<String> {
    let mut version_counts: HashMap<String, Vec<String>> = HashMap::new();

    // Group machines by their content
    for (machine_id, content) in all_versions {
        version_counts
            .entry(content.clone())
            .or_insert_with(Vec::new)
            .push(machine_id.clone());
    }

    // Find the most common version
    let mut max_count = 0;
    let mut consensus_content = String::new();
    let mut current_machine_content: Option<String> = None;

    for (content, machines) in version_counts {
        if machines.contains(&current_machine_id.to_string()) {
            current_machine_content = Some(content.clone());
        }

        if machines.len() > max_count {
            max_count = machines.len();
            consensus_content = content;
        } else if machines.len() == max_count {
            // Tie: prefer current machine's version if it's one of the tied versions
            if machines.contains(&current_machine_id.to_string()) {
                consensus_content = content;
            }
        }
    }

    if max_count == 0 {
        return Err(crate::error::DriftersError::Config(
            "No consensus version found".to_string(),
        ));
    }

    // If there's a clear majority (more than half), use it
    let total_machines = all_versions.len();
    if max_count > total_machines / 2 {
        return Ok(consensus_content);
    }

    // Otherwise, log a warning and use the consensus
    log::warn!(
        "No clear consensus for merge ({}/{} machines agree). Using majority version.",
        max_count,
        total_machines
    );

    // If current machine has a version and there's no clear winner, prefer current
    if let Some(current_content) = current_machine_content {
        if max_count <= total_machines / 2 {
            log::info!("Preferring current machine's version due to tie");
            return Ok(current_content);
        }
    } else {
        log::warn!(
            "Current machine '{}' has no version in tie-break; result is non-deterministic. \
             Run `drifters push` to register this machine's version.",
            current_machine_id
        );
    }

    Ok(consensus_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_version() {
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), "content".to_string());

        let result =
            intelligent_merge(&versions, "machine1", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "content");
    }

    #[test]
    fn test_identical_versions() {
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), "same".to_string());
        versions.insert("machine2".to_string(), "same".to_string());
        versions.insert("machine3".to_string(), "same".to_string());

        let result =
            intelligent_merge(&versions, "machine1", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "same");
    }

    #[test]
    fn test_consensus_majority() {
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), "version_a".to_string());
        versions.insert("machine2".to_string(), "version_a".to_string());
        versions.insert("machine3".to_string(), "version_b".to_string());

        let result =
            intelligent_merge(&versions, "machine1", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "version_a"); // Majority wins
    }

    #[test]
    fn test_tie_prefers_current_machine() {
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), "my_version".to_string());
        versions.insert("machine2".to_string(), "other_version".to_string());

        let result =
            intelligent_merge(&versions, "machine1", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "my_version"); // Tie, prefer current machine
    }
}
