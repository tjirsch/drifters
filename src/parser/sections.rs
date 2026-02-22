use crate::error::{DriftersError, Result};

/// Extract syncable content (everything EXCEPT exclude sections)
/// Returns the content that should be synced to other machines
pub fn extract_syncable_content(content: &str, comment_syntax: &str) -> Result<Option<String>> {
    let exclude_start = format!("{} drifters::exclude::start", comment_syntax);
    let exclude_stop = format!("{} drifters::exclude::stop", comment_syntax);

    let mut result = String::new();
    let mut in_exclude_block = false;
    let mut found_any_tags = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with(&exclude_start) {
            in_exclude_block = true;
            found_any_tags = true;
            // Include the tag itself for reconstruction
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if trimmed.starts_with(&exclude_stop) {
            in_exclude_block = false;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if !in_exclude_block {
            result.push_str(line);
            result.push('\n');
        }
    }

    // A start tag with no matching stop tag is always a user error: it would
    // silently exclude everything from the unclosed tag to EOF.
    if in_exclude_block {
        return Err(DriftersError::Config(
            "unclosed drifters::exclude::start block \
             (missing drifters::exclude::stop)"
                .to_string(),
        ));
    }

    if found_any_tags {
        Ok(Some(result))
    } else {
        // No tags found, sync entire file
        Ok(None)
    }
}

/// Merge synced content back into local file
/// Preserves local exclude sections, replaces everything else
pub fn merge_synced_content(
    local_content: &str,
    synced_content: &str,
    comment_syntax: &str,
) -> Result<String> {
    let exclude_start = format!("{} drifters::exclude::start", comment_syntax);
    let exclude_stop = format!("{} drifters::exclude::stop", comment_syntax);

    // Extract local exclude sections with their positions
    let local_excludes = extract_exclude_sections(local_content, &exclude_start, &exclude_stop)?;

    let mut result = String::new();
    let mut in_exclude_block = false;
    let mut exclude_index = 0;

    for line in synced_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with(&exclude_start) {
            // Use local exclude section if it exists
            if let Some(local_exclude) = local_excludes.get(exclude_index) {
                result.push_str(local_exclude);
                exclude_index += 1;
            } else {
                // No local version, include the synced exclude section
                result.push_str(line);
                result.push('\n');
            }
            in_exclude_block = true;
            continue;
        }

        if trimmed.starts_with(&exclude_stop) {
            in_exclude_block = false;
            // Skip stop tag if we already included it with local content
            if exclude_index > 0 && local_excludes.get(exclude_index - 1).is_some() {
                continue;
            }
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if !in_exclude_block {
            result.push_str(line);
            result.push('\n');
        }
        // Skip lines inside exclude blocks (they come from local_excludes)
    }

    Ok(result)
}

/// Extract exclude sections from content
fn extract_exclude_sections(
    content: &str,
    start_tag: &str,
    stop_tag: &str,
) -> Result<Vec<String>> {
    let mut sections = Vec::new();
    let mut current_section = String::new();
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with(start_tag) {
            in_section = true;
            current_section.clear();
            current_section.push_str(line);
            current_section.push('\n');
            continue;
        }

        if trimmed.starts_with(stop_tag) {
            current_section.push_str(line);
            current_section.push('\n');
            sections.push(current_section.clone());
            in_section = false;
            continue;
        }

        if in_section {
            current_section.push_str(line);
            current_section.push('\n');
        }
    }

    // A dangling start tag means the partial section would be silently dropped,
    // causing the local exclude content to disappear on the next pull.
    if in_section {
        return Err(DriftersError::Config(
            "unclosed drifters::exclude::start block \
             (missing drifters::exclude::stop)"
                .to_string(),
        ));
    }

    Ok(sections)
}

/// Detect comment syntax from file extension
pub fn detect_comment_syntax(filename: &str) -> &str {
    // Check for special filenames first
    if filename.contains("vimrc") || filename.ends_with(".vim") {
        return "\"";
    }

    // Use Path::extension() rather than split('.').last() so that compound
    // extensions like "init.lua.bak" return "bak" (correct) and not "lua".
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        // Shell scripts, Python, Ruby, YAML, TOML
        "sh" | "bash" | "zsh" | "py" | "rb" | "yaml" | "yml" | "toml" | "conf" => "#",
        // JavaScript, TypeScript, C, C++, Rust, Go, Java
        "js" | "ts" | "jsx" | "tsx" | "c" | "cpp" | "h" | "hpp" | "rs" | "go" | "java" => "//",
        // Lua
        "lua" => "--",
        // Vim
        "vim" => "\"",
        // SQL
        "sql" => "--",
        // Default to # for unknown files
        _ => "#",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_syncable_content_exclude() {
        let content = r#"
export SHARED="shared"

# drifters::exclude::start
export LOCAL_ONLY="local"
alias local_alias="foo"
# drifters::exclude::stop

export ANOTHER_SHARED="also shared"
"#;

        let result = extract_syncable_content(content, "#").unwrap();
        assert!(result.is_some());
        let synced = result.unwrap();
        assert!(synced.contains("export SHARED"));
        assert!(synced.contains("export ANOTHER_SHARED"));
        assert!(synced.contains("# drifters::exclude::start"));
        assert!(synced.contains("# drifters::exclude::stop"));
        assert!(!synced.contains("export LOCAL_ONLY"));
        assert!(!synced.contains("alias local_alias"));
    }

    #[test]
    fn test_no_tags() {
        let content = "export EDITOR=\"nvim\"\nalias g=\"git\"";
        let result = extract_syncable_content(content, "#").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_merge_synced_content() {
        let local = r#"
export SHARED="old_value"

# drifters::exclude::start
export LOCAL="my_local_value"
# drifters::exclude::stop

export OTHER="old_other"
"#;

        let synced = r#"
export SHARED="new_value"

# drifters::exclude::start
# drifters::exclude::stop

export OTHER="new_other"
"#;

        let result = merge_synced_content(local, synced, "#").unwrap();
        assert!(result.contains("export SHARED=\"new_value\""));
        assert!(result.contains("export OTHER=\"new_other\""));
        assert!(result.contains("export LOCAL=\"my_local_value\""));
    }

    #[test]
    fn test_multiple_exclude_sections() {
        let content = r#"
export SHARED1="shared"

# drifters::exclude::start
export LOCAL1="local"
# drifters::exclude::stop

export SHARED2="shared"

# drifters::exclude::start
export LOCAL2="local"
# drifters::exclude::stop
"#;

        let result = extract_syncable_content(content, "#").unwrap();
        assert!(result.is_some());
        let synced = result.unwrap();
        assert!(synced.contains("SHARED1"));
        assert!(synced.contains("SHARED2"));
        assert!(!synced.contains("LOCAL1"));
        assert!(!synced.contains("LOCAL2"));
    }

    #[test]
    fn test_leading_whitespace_tags_are_recognized() {
        // Tags with leading whitespace (e.g. inside indented blocks) must be detected
        let content = "export SHARED=\"shared\"\n  # drifters::exclude::start\nexport LOCAL=\"local\"\n  # drifters::exclude::stop\nexport OTHER=\"other\"\n";
        let result = extract_syncable_content(content, "#").unwrap();
        assert!(result.is_some());
        let synced = result.unwrap();
        assert!(synced.contains("export SHARED"));
        assert!(synced.contains("export OTHER"));
        assert!(!synced.contains("export LOCAL"));
    }

    #[test]
    fn test_inline_tags_are_ignored() {
        // Tags that appear after other content on the same line must NOT be treated as section delimiters
        let content = "export VAR=\"val\"  # drifters::exclude::start\nexport OTHER=\"other\"\n";
        let result = extract_syncable_content(content, "#").unwrap();
        // No tags should have been detected, so the whole file is synced
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_comment_syntax() {
        assert_eq!(detect_comment_syntax("test.sh"), "#");
        assert_eq!(detect_comment_syntax("config.py"), "#");
        assert_eq!(detect_comment_syntax("app.js"), "//");
        assert_eq!(detect_comment_syntax("main.rs"), "//");
        assert_eq!(detect_comment_syntax("init.lua"), "--");
        assert_eq!(detect_comment_syntax(".vimrc"), "\"");
    }

    #[test]
    fn test_unclosed_exclude_block_extract() {
        // A start tag with no matching stop tag must be an error, not a silent
        // "exclude everything to EOF".
        let content = "export SHARED=\"shared\"\n\
                       # drifters::exclude::start\n\
                       export LOCAL=\"local\"\n";
        let result = extract_syncable_content(content, "#");
        assert!(result.is_err(), "expected Err for unclosed exclude block");
        assert!(result.unwrap_err().to_string().contains("unclosed"));
    }

    #[test]
    fn test_unclosed_exclude_block_merge() {
        // extract_exclude_sections is called inside merge_synced_content; an
        // unclosed block in the local content must surface as an error so the
        // caller knows the local file is malformed rather than silently dropping
        // the partial exclude section.
        let local = "# drifters::exclude::start\nexport LOCAL=\"local\"\n";
        let synced = "export SHARED=\"shared\"\n\
                      # drifters::exclude::start\n\
                      # drifters::exclude::stop\n";
        let result = merge_synced_content(local, synced, "#");
        assert!(result.is_err(), "expected Err for unclosed exclude block in local");
        assert!(result.unwrap_err().to_string().contains("unclosed"));
    }
}
