use crate::error::{DriftersError, Result};
use std::path::PathBuf;

const EMPTY_FILE_THRESHOLD: u64 = 10; // bytes
const SIZE_RATIO_WARNING: f64 = 10.0; // Warn if file is 10x smaller

/// Check if it's safe to push a local file
/// Returns Ok(true) if safe, Ok(false) if user should be warned
pub fn check_file_safety(local_path: &PathBuf, repo_path: &PathBuf) -> Result<bool> {
    if !local_path.exists() {
        return Err(DriftersError::FileNotFound(local_path.clone()));
    }

    let local_metadata = std::fs::metadata(local_path)?;
    let local_size = local_metadata.len();

    // Check if local file is empty or very small
    if local_size < EMPTY_FILE_THRESHOLD {
        // Check if repo version exists and is larger
        if let Ok(repo_metadata) = std::fs::metadata(repo_path) {
            let repo_size = repo_metadata.len();

            if repo_size > EMPTY_FILE_THRESHOLD {
                log::warn!(
                    "Local file {:?} is {} bytes but repo version is {} bytes",
                    local_path,
                    local_size,
                    repo_size
                );
                return Ok(false); // Not safe, warn user
            }
        }
    }

    // Check if local file is significantly smaller than repo version
    if let Ok(repo_metadata) = std::fs::metadata(repo_path) {
        let repo_size = repo_metadata.len();

        if repo_size > 0 && local_size > 0 {
            let ratio = repo_size as f64 / local_size as f64;

            if ratio > SIZE_RATIO_WARNING {
                log::warn!(
                    "Local file {:?} is {}x smaller than repo version ({} vs {} bytes)",
                    local_path,
                    ratio,
                    local_size,
                    repo_size
                );
                return Ok(false); // Not safe, warn user
            }
        }
    }

    Ok(true)
}

/// Confirm with user before proceeding with potentially dangerous operation
pub fn confirm_operation(message: &str, default_yes: bool) -> Result<bool> {
    use std::io::{self, Write};

    let prompt = if default_yes {
        format!("{} [Y/n]: ", message)
    } else {
        format!("{} [y/N]: ", message)
    };

    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let answer = input.trim().to_lowercase();

    let confirmed = match answer.as_str() {
        "" => default_yes,
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default_yes,
    };

    Ok(confirmed)
}
