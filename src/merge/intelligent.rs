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

/// Find the version that appears most frequently across machines.
///
/// Tie-breaking rules (in priority order):
/// 1. Prefer the current machine's version if it is among the leaders.
/// 2. Otherwise, pick the lexicographically smallest content string.
///    This is an arbitrary but *stable* rule — the same inputs always
///    produce the same winner regardless of HashMap iteration order.
fn find_consensus_version(
    all_versions: &HashMap<String, String>,
    current_machine_id: &str,
) -> Result<String> {
    // Group machines by their content
    let mut version_counts: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_machine_content: Option<String> = None;

    for (machine_id, content) in all_versions {
        if machine_id == current_machine_id {
            current_machine_content = Some(content.clone());
        }
        version_counts
            .entry(content.clone())
            .or_default()
            .push(machine_id.clone());
    }

    // Find the highest vote count
    let max_count = version_counts
        .values()
        .map(|machines| machines.len())
        .max()
        .unwrap_or(0);

    if max_count == 0 {
        return Err(crate::error::DriftersError::Config(
            "No consensus version found".to_string(),
        ));
    }

    // Collect all versions tied at the top
    let mut top_versions: Vec<&String> = version_counts
        .iter()
        .filter(|(_, machines)| machines.len() == max_count)
        .map(|(content, _)| content)
        .collect();

    // Tie-break 1: prefer the current machine's version if it is in the leaders
    if let Some(ref cur) = current_machine_content {
        if top_versions.contains(&cur) {
            let total_machines = all_versions.len();
            if max_count > total_machines / 2 {
                return Ok(cur.clone());
            }
            // No clear majority but current machine is in the tie — prefer it
            log::warn!(
                "No clear consensus ({}/{} machines agree). \
                 Preferring current machine's version.",
                max_count,
                total_machines
            );
            return Ok(cur.clone());
        }
    }

    // Tie-break 2: lexicographically smallest content — deterministic, stable
    top_versions.sort();
    let winner = top_versions.into_iter().next().unwrap().clone();

    let total_machines = all_versions.len();
    if max_count > total_machines / 2 {
        return Ok(winner);
    }

    log::warn!(
        "No clear consensus ({}/{} machines agree) and current machine '{}' has no version \
         among the leaders. Using lexicographically smallest tied version as stable tie-break. \
         Run `drifters push-app` to register this machine's version.",
        max_count,
        total_machines,
        current_machine_id
    );

    Ok(winner)
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

    #[test]
    fn test_tie_without_current_machine_is_deterministic() {
        // Neither "aaa" nor "bbb" belongs to the current machine.
        // The winner must always be the same regardless of HashMap insertion order.
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), "bbb".to_string());
        versions.insert("machine2".to_string(), "aaa".to_string());

        let result1 =
            intelligent_merge(&versions, "machine3", "test.txt", &Default::default()).unwrap();

        // Reverse insertion order
        let mut versions2 = HashMap::new();
        versions2.insert("machine2".to_string(), "aaa".to_string());
        versions2.insert("machine1".to_string(), "bbb".to_string());

        let result2 =
            intelligent_merge(&versions2, "machine3", "test.txt", &Default::default()).unwrap();

        assert_eq!(result1, result2); // Must be the same
        assert_eq!(result1, "aaa");   // Lexicographically smallest wins
    }
}
