use crate::config::LocalConfig;
use crate::error::Result;
use serde::Deserialize;

const REPO: &str = "tjirsch/drifters";
const API_URL: &str = "https://api.github.com/repos";

/// A release asset (file attached to a GitHub release)
#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Top-level release metadata returned by the GitHub Releases API
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    assets: Vec<Asset>,
}

pub fn check_update_available(
    client: &reqwest::blocking::Client,
) -> Result<Option<(String, String)>> {
    let url = format!("{}/{}/releases/latest", API_URL, REPO);
    let response = client.get(&url).send()?;
    if !response.status().is_success() {
        return Ok(None);
    }
    let release: Release = response.json()?;
    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let current = env!("CARGO_PKG_VERSION");
    if compare_versions(current, &latest_version) < 0 {
        Ok(Some((latest_version, release.html_url)))
    } else {
        Ok(None)
    }
}

pub fn maybe_check_for_updates(config: &mut LocalConfig) -> Result<()> {
    let freq = config.self_update_frequency.as_str();
    if freq == "never" {
        return Ok(());
    }
    if freq == "daily" {
        if let Some(last) = config.last_update_check {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now.saturating_sub(last) < 86400 {
                return Ok(());
            }
        }
    }
    let client = reqwest::blocking::Client::builder()
        .user_agent("drifters-update-checker")
        .build()?;
    let update = check_update_available(&client)?;
    if freq == "daily" {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        config.last_update_check = Some(now);
        let _ = config.save();
    }
    if let Some((version, url)) = update {
        println!(
            "⚠️  Update available: {} (current: {}). Run `drifters self-update` to install. {}",
            version,
            env!("CARGO_PKG_VERSION"),
            url
        );
    }
    Ok(())
}

/// Check for and optionally install a new release.
///
/// `check_only`          – print update info but do not download or install.
/// `skip_checksum`       – skip SHA-256 verification even if no sidecar exists.
///                         Use only if you trust the download channel and the release
///                         predates checksum support.
/// `no_download_readme`  – skip downloading the README after a successful update.
/// `no_open_readme`      – download README but do not open it.
/// `preferred_editor`    – editor to use when opening README (see `open_file`).
pub fn run_self_update(
    check_only: bool,
    skip_checksum: bool,
    no_download_readme: bool,
    no_open_readme: bool,
    preferred_editor: Option<&str>,
) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    let client = reqwest::blocking::Client::builder()
        .user_agent("drifters-update-checker")
        .build()?;

    let url = format!("{}/{}/releases/latest", API_URL, REPO);
    let response = client.get(&url).send()?;

    if !response.status().is_success() {
        eprintln!("Failed to fetch release information from GitHub");
        eprintln!("Repository: {}", REPO);
        eprintln!("URL: {}", url);
        eprintln!("Status: {}", response.status());
        return Err(crate::error::DriftersError::Config(
            "Unable to check for updates".to_string(),
        ));
    }

    let release: Release = response.json()?;
    let latest_version = release.tag_name.trim_start_matches('v');
    println!("Latest version: {}", latest_version);

    if compare_versions(current_version, latest_version) < 0 {
        println!("\n⚠️  A new version is available!");
        println!("   Current: {}", current_version);
        println!("   Latest:  {}", latest_version);
        println!("   Release: {}", release.html_url);
        if check_only {
            println!("\nRun `drifters self-update` to install.");
            return Ok(());
        }
        println!("\n📥 Installing update...");

        let installer_url = format!(
            "https://github.com/{}/releases/latest/download/drifters-installer.sh",
            REPO
        );

        // ── Download installer as raw bytes ──────────────────────────────────
        let installer_bytes = client.get(&installer_url).send()?.bytes()?;

        // ── Checksum verification ─────────────────────────────────────────────
        // Look for a SHA-256 sidecar uploaded alongside the installer.
        let checksum_asset = release
            .assets
            .iter()
            .find(|a| a.name == "drifters-installer.sh.sha256");

        match checksum_asset {
            Some(asset) => {
                // Sidecar found — download and compare
                let expected_raw = client
                    .get(&asset.browser_download_url)
                    .send()?
                    .text()?;
                // sha256sum output format: "<hex>  <filename>"
                let expected = expected_raw
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase();

                use sha2::{Digest, Sha256};
                let actual = hex::encode(Sha256::digest(&installer_bytes));

                if actual != expected {
                    return Err(crate::error::DriftersError::Config(format!(
                        "Checksum mismatch — installer may have been tampered with.\n\
                         Expected: {}\n\
                         Got:      {}\n\
                         Aborting. Download the release manually from {}",
                        expected, actual, release.html_url
                    )));
                }
                println!("✅ Checksum verified");
            }
            None if skip_checksum => {
                // No sidecar but user explicitly opted in — warn and continue
                eprintln!(
                    "⚠️  No checksum file found in this release. \
                     Proceeding without verification (--skip-checksum)."
                );
            }
            None => {
                // No sidecar and no explicit bypass — refuse to install
                return Err(crate::error::DriftersError::Config(
                    "No checksum file (drifters-installer.sh.sha256) found in this release.\n\
                     Cannot verify installer integrity. Aborting.\n\
                     If you are confident in the download, re-run with --skip-checksum."
                        .to_string(),
                ));
            }
        }

        // ── Write and execute ─────────────────────────────────────────────────
        let temp_file = std::env::temp_dir()
            .join(format!("drifters-installer-{}.sh", std::process::id()));
        std::fs::write(&temp_file, &installer_bytes)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temp_file, std::fs::Permissions::from_mode(0o755))?;

            let status = std::process::Command::new("sh").arg(&temp_file).status()?;
            let _ = std::fs::remove_file(&temp_file);

            if status.success() {
                println!("✅ Update installed successfully!");
                println!("   Please restart your terminal or run: source ~/.profile");

                if !no_download_readme {
                    let open_editor = if no_open_readme { None } else { preferred_editor };
                    match crate::cli::open_readme::run_open_readme(open_editor) {
                        Ok(()) => {}
                        Err(e) => eprintln!("⚠️  Could not download README: {}", e),
                    }
                }
            } else {
                eprintln!("Failed to run installer script");
                eprintln!("Installer URL: {}", installer_url);
                eprintln!("Exit code: {:?}", status.code());
                return Err(crate::error::DriftersError::Config(
                    "Installer script execution failed".to_string(),
                ));
            }
        }

        #[cfg(windows)]
        {
            let _ = std::fs::remove_file(&temp_file);
            return Err(crate::error::DriftersError::Config(
                "Automatic installation on Windows is not yet supported. \
                 Please download and run the installer manually."
                    .to_string(),
            ));
        }
    } else {
        println!("✅ You are running the latest version!");
    }

    Ok(())
}

fn compare_versions(v1: &str, v2: &str) -> i32 {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .map(|s| {
                s.parse::<u32>().unwrap_or_else(|_| {
                    log::debug!(
                        "Failed to parse version segment '{}' in '{}', treating as 0",
                        s,
                        v
                    );
                    0
                })
            })
            .collect()
    };
    let v1_parts = parse_version(v1);
    let v2_parts = parse_version(v2);
    let max_len = v1_parts.len().max(v2_parts.len());
    for i in 0..max_len {
        let a = v1_parts.get(i).copied().unwrap_or(0);
        let b = v2_parts.get(i).copied().unwrap_or(0);
        if a < b {
            return -1;
        }
        if a > b {
            return 1;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert_eq!(compare_versions("1.0.0", "1.0.1"), -1);
        assert_eq!(compare_versions("1.0.1", "1.0.0"), 1);
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert_eq!(compare_versions("1.0", "1.0.0"), 0);
        assert_eq!(compare_versions("1.2.3", "1.10.0"), -1);
        assert_eq!(compare_versions("2.0.0", "1.99.99"), 1);
    }
}
