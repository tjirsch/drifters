use crate::config::LocalConfig;
use crate::error::Result;
use serde::Deserialize;

const REPO: &str = "tjirsch/drifters";
const API_URL: &str = "https://api.github.com/repos";

pub fn check_update_available(
    client: &reqwest::blocking::Client,
) -> Result<Option<(String, String)>> {
    let url = format!("{}/{}/releases/latest", API_URL, REPO);
    let response = client.get(&url).send()?;
    if !response.status().is_success() {
        return Ok(None);
    }
    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
        html_url: String,
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
        if let Some(ref last) = config.last_update_check {
            let last_ts: u64 = last.parse().unwrap_or(0);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now.saturating_sub(last_ts) < 86400 {
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
        config.last_update_check = Some(now.to_string());
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

pub fn run_self_update(check_only: bool) -> Result<()> {
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
            "Unable to check for updates".to_string()
        ));
    }

    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
        html_url: String,
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
        let installer_script = client.get(&installer_url).send()?.text()?;
        let temp_file = std::env::temp_dir()
            .join(format!("drifters-installer-{}.sh", std::process::id()));
        std::fs::write(&temp_file, installer_script)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temp_file, std::fs::Permissions::from_mode(0o755))?;

            let status = std::process::Command::new("sh").arg(&temp_file).status()?;
            let _ = std::fs::remove_file(&temp_file);

            if status.success() {
                println!("✅ Update installed successfully!");
                println!("   Please restart your terminal or run: source ~/.profile");
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
                "Automatic installation on Windows is not yet supported. Please download and run the installer manually.".to_string(),
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
                    log::debug!("Failed to parse version segment '{}' in '{}', treating as 0", s, v);
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
