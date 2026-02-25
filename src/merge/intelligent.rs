use crate::config::AppConfig;
use crate::error::Result;
use crate::git::MachineVersion;
use std::collections::HashMap;

/// Select the version to apply when merging configs from multiple machines.
///
/// Strategy: **last-write-wins** using git commit timestamps.
///
/// * Each version's effective timestamp is `committed_at.unwrap_or(0)`.
///   Files with no git history (legacy repos) are treated as oldest.
/// * The version with the highest effective timestamp wins.
/// * If multiple versions share the same maximum timestamp (e.g. two machines
///   pushed within the same second), tiebreak by preferring the current
///   machine's version, then the lexicographically smallest content string
///   (deterministic and stable regardless of HashMap iteration order).
pub fn intelligent_merge(
    all_versions: &HashMap<String, MachineVersion>,
    current_machine_id: &str,
    _filename: &str,
    _app_config: &AppConfig,
) -> Result<String> {
    if all_versions.is_empty() {
        return Err(crate::error::DriftersError::Config(
            "No versions available to merge".to_string(),
        ));
    }

    // Single version — nothing to decide
    if all_versions.len() == 1 {
        return Ok(all_versions.values().next().unwrap().content.clone());
    }

    // All versions identical — use any
    let first_content = &all_versions.values().next().unwrap().content;
    if all_versions.values().all(|v| &v.content == first_content) {
        return Ok(first_content.clone());
    }

    // Find the maximum effective timestamp across all versions
    let max_ts = all_versions
        .values()
        .map(|v| v.committed_at.unwrap_or(0))
        .max()
        .unwrap_or(0);

    // Collect all versions tied at that timestamp (machine_id → content)
    let mut winners: Vec<(&str, &str)> = all_versions
        .iter()
        .filter(|(_, v)| v.committed_at.unwrap_or(0) == max_ts)
        .map(|(id, v)| (id.as_str(), v.content.as_str()))
        .collect();

    // Exactly one winner — done
    if winners.len() == 1 {
        log::debug!(
            "Last-write-wins: '{}' has the most recent commit (ts={})",
            winners[0].0,
            max_ts
        );
        return Ok(winners[0].1.to_owned());
    }

    // Tiebreak 1: prefer the current machine if it is among the winners
    if let Some((_, content)) = winners.iter().find(|(id, _)| *id == current_machine_id) {
        log::warn!(
            "Timestamp tie ({} machines at ts={}). Preferring current machine '{}'.",
            winners.len(),
            max_ts,
            current_machine_id
        );
        return Ok(content.to_string());
    }

    // Tiebreak 2: lexicographically smallest content (stable, deterministic)
    winners.sort_by_key(|(_, content)| *content);
    log::warn!(
        "Timestamp tie ({} machines at ts={}), current machine '{}' not among them. \
         Using lexicographically smallest content as stable tiebreak.",
        winners.len(),
        max_ts,
        current_machine_id
    );
    Ok(winners[0].1.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(content: &str, ts: Option<u64>) -> MachineVersion {
        MachineVersion {
            content: content.to_string(),
            committed_at: ts,
        }
    }

    #[test]
    fn test_single_version() {
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), mv("content", None));

        let result =
            intelligent_merge(&versions, "machine1", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "content");
    }

    #[test]
    fn test_identical_versions() {
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), mv("same", None));
        versions.insert("machine2".to_string(), mv("same", None));
        versions.insert("machine3".to_string(), mv("same", None));

        let result =
            intelligent_merge(&versions, "machine1", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "same");
    }

    #[test]
    fn test_newest_timestamp_wins_over_majority() {
        // Machine A has a new version (ts=100); B and C have the old version (ts=50).
        // Last-write-wins: A's version must win even though it is outnumbered 1 vs 2.
        let mut versions = HashMap::new();
        versions.insert("machine_a".to_string(), mv("new_version", Some(100)));
        versions.insert("machine_b".to_string(), mv("old_version", Some(50)));
        versions.insert("machine_c".to_string(), mv("old_version", Some(50)));

        let result =
            intelligent_merge(&versions, "machine_b", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "new_version");
    }

    #[test]
    fn test_none_timestamp_loses_to_any_real_timestamp() {
        let mut versions = HashMap::new();
        versions.insert("legacy".to_string(), mv("legacy_content", None)); // ts = 0
        versions.insert("modern".to_string(), mv("modern_content", Some(1))); // ts = 1

        let result =
            intelligent_merge(&versions, "legacy", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "modern_content");
    }

    #[test]
    fn test_timestamp_tie_prefers_current_machine() {
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), mv("my_version", Some(42)));
        versions.insert("machine2".to_string(), mv("other_version", Some(42)));

        let result =
            intelligent_merge(&versions, "machine1", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "my_version");
    }

    #[test]
    fn test_timestamp_tie_without_current_machine_is_deterministic() {
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), mv("bbb", Some(10)));
        versions.insert("machine2".to_string(), mv("aaa", Some(10)));

        let result1 =
            intelligent_merge(&versions, "machine3", "test.txt", &Default::default()).unwrap();

        // Reverse insertion order — result must be identical
        let mut versions2 = HashMap::new();
        versions2.insert("machine2".to_string(), mv("aaa", Some(10)));
        versions2.insert("machine1".to_string(), mv("bbb", Some(10)));

        let result2 =
            intelligent_merge(&versions2, "machine3", "test.txt", &Default::default()).unwrap();

        assert_eq!(result1, result2);
        assert_eq!(result1, "aaa"); // lexicographically smallest wins
    }

    #[test]
    fn test_all_none_timestamps_tiebreak_is_deterministic() {
        // Legacy repo: no timestamps at all. Tiebreak: current machine → lex smallest.
        let mut versions = HashMap::new();
        versions.insert("machine1".to_string(), mv("zzz", None));
        versions.insert("machine2".to_string(), mv("aaa", None));

        // machine3 is current machine and is not in the versions — lex wins
        let result =
            intelligent_merge(&versions, "machine3", "test.txt", &Default::default()).unwrap();
        assert_eq!(result, "aaa");
    }
}
