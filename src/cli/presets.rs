use crate::config::{AppConfig, LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, EphemeralRepoGuard};
use serde::Deserialize;
use std::io::{self, Write};

// Parse repository from Cargo.toml at compile time
// Expected format: https://github.com/owner/repo
const CARGO_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

#[derive(Deserialize)]
struct GitHubContent {
    name: String,
    #[serde(rename = "type")]
    file_type: String,
}

/// Parse GitHub owner and repo from repository URL
fn parse_github_repo() -> Result<(String, String)> {
    let url = CARGO_REPOSITORY;

    // Remove https://github.com/ prefix
    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .ok_or_else(|| {
            DriftersError::Config(format!(
                "Repository URL in Cargo.toml is not a GitHub URL: {}",
                url
            ))
        })?;

    // Split into owner/repo
    let parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();
    if parts.len() < 2 {
        return Err(DriftersError::Config(format!(
            "Invalid GitHub repository URL: {}",
            url
        )));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub fn list_presets() -> Result<()> {
    println!("Fetching available presets from GitHub...\n");

    let (owner, repo) = parse_github_repo()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/presets",
        owner, repo
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("drifters-cli")
        .build()?;

    let response = client.get(&url).send()?;

    if !response.status().is_success() {
        eprintln!("Failed to fetch presets from GitHub");
        eprintln!("Repository: https://github.com/{}/{}", owner, repo);
        eprintln!("URL: {}", url);
        eprintln!("Status: {}", response.status());
        return Err(DriftersError::Config(format!(
            "Unable to access presets from https://github.com/{}/{}",
            owner, repo
        )));
    }

    let contents: Vec<GitHubContent> = response.json()?;

    let presets: Vec<String> = contents
        .into_iter()
        .filter(|item| item.file_type == "file" && item.name.ends_with(".toml"))
        .map(|item| item.name.trim_end_matches(".toml").to_string())
        .filter(|name| name != "README")
        .collect();

    if presets.is_empty() {
        println!("No presets found");
        return Ok(());
    }

    println!("Available presets:");
    for preset in &presets {
        println!("  - {}", preset);
    }

    println!("\nTo load a preset:");
    println!("  drifters load-preset <name>");
    println!("\nExample:");
    println!("  drifters load-preset zed");

    Ok(())
}

pub fn load_preset(preset_name: String) -> Result<()> {
    println!("Loading preset '{}' from GitHub...", preset_name);

    let (owner, repo) = parse_github_repo()?;
    let file_path = format!("presets/{}.toml", preset_name);
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        owner, repo, file_path
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("drifters-cli")
        .build()?;

    let response = client.get(&url).send()?;

    if !response.status().is_success() {
        eprintln!("Failed to fetch preset '{}' from GitHub", preset_name);
        eprintln!("Repository: https://github.com/{}/{}", owner, repo);
        eprintln!("File: {}", file_path);
        eprintln!("URL: {}", url);
        eprintln!("Status: {}", response.status());
        return Err(DriftersError::Config(format!(
            "Preset '{}' not found or inaccessible",
            preset_name
        )));
    }

    #[derive(Deserialize)]
    struct FileContent {
        content: String,
    }

    let file_content: FileContent = response.json()?;

    // Decode base64 content (GitHub API returns file content as base64)
    use base64::Engine;
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(file_content.content.replace('\n', ""))
        .map_err(|e| {
            DriftersError::Config(format!("Failed to decode base64 content: {}", e))
        })?;

    let preset_content = String::from_utf8(decoded_bytes).map_err(|e| {
        DriftersError::Config(format!("Failed to decode UTF-8 content: {}", e))
    })?;

    // Parse the preset
    let preset_rules: SyncRules = toml::from_str(&preset_content)?;

    // The preset should contain exactly one app with the same name
    let app_config = preset_rules
        .apps
        .get(&preset_name)
        .ok_or_else(|| {
            DriftersError::Config(format!(
                "Preset '{}' does not contain app definition for '{}'",
                preset_name, preset_name
            ))
        })?
        .clone();

    // Load local config and repo
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // Load current sync rules
    let mut rules = SyncRules::load(repo_path)?;

    // Check if app already exists
    let is_update = rules.apps.contains_key(&preset_name);

    // Update or add the app
    rules.apps.insert(preset_name.clone(), app_config);

    // Save rules
    rules.save(repo_path)?;

    let action = if is_update { "Updated" } else { "Added" };
    println!("\n✓ {} '{}' from preset", action, preset_name);

    // Commit and push
    println!("\nCommitting changes...");
    let message = format!("{} {} app from preset", action, preset_name);
    commit_and_push(repo_path, &message)?;

    println!("✓ Changes committed and pushed");
    println!(
        "\nRun 'drifters merge-app {}' to apply the new rules",
        preset_name
    );

    Ok(())
}

pub fn discover_presets() -> Result<()> {
    let (owner, repo) = parse_github_repo()?;

    // ── 1. Connect to repo and load current rules ─────────────────────────
    let config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();
    let mut rules = SyncRules::load(repo_path)?;

    // ── 2. Fetch the list of available presets ────────────────────────────
    println!("Fetching available presets from GitHub...");
    let client = reqwest::blocking::Client::builder()
        .user_agent("drifters-cli")
        .build()?;

    let list_url = format!(
        "https://api.github.com/repos/{}/{}/contents/presets",
        owner, repo
    );
    let list_resp = client.get(&list_url).send()?;
    if !list_resp.status().is_success() {
        return Err(DriftersError::Config(format!(
            "Unable to fetch presets from https://github.com/{}/{}",
            owner, repo
        )));
    }
    let contents: Vec<GitHubContent> = list_resp.json()?;
    let preset_names: Vec<String> = contents
        .into_iter()
        .filter(|i| i.file_type == "file" && i.name.ends_with(".toml"))
        .map(|i| i.name.trim_end_matches(".toml").to_string())
        .filter(|n| n != "README")
        .collect();

    if preset_names.is_empty() {
        println!("No presets found in the repository.");
        return Ok(());
    }

    println!("Checking {} preset(s) for installed apps...", preset_names.len());

    // ── 3. For each preset not already configured, check if its files exist ─
    let home = dirs::home_dir().unwrap_or_default();
    let mut detected: Vec<(String, AppConfig)> = Vec::new();
    let mut already_configured = 0usize;

    #[derive(Deserialize)]
    struct FileContent {
        content: String,
    }

    for preset_name in &preset_names {
        if rules.apps.contains_key(preset_name) {
            already_configured += 1;
            continue;
        }

        // Fetch the preset TOML
        let file_url = format!(
            "https://api.github.com/repos/{}/{}/contents/presets/{}.toml",
            owner, repo, preset_name
        );
        let resp = match client.get(&file_url).send() {
            Ok(r) if r.status().is_success() => r,
            _ => continue, // skip on any fetch error
        };
        let file_content: FileContent = match resp.json() {
            Ok(fc) => fc,
            Err(_) => continue,
        };

        // Decode base64
        use base64::Engine;
        let decoded_bytes = match base64::engine::general_purpose::STANDARD
            .decode(file_content.content.replace('\n', ""))
        {
            Ok(b) => b,
            Err(_) => continue,
        };
        let toml_str = match String::from_utf8(decoded_bytes) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let preset_rules: SyncRules = match toml::from_str(&toml_str) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let app_config = match preset_rules.apps.get(preset_name).cloned() {
            Some(c) => c,
            None => continue,
        };

        // Collect include patterns for the current OS
        let mut patterns = app_config.include.clone();
        match std::env::consts::OS {
            "macos"   => patterns.extend(app_config.include_macos.clone()),
            "linux"   => patterns.extend(app_config.include_linux.clone()),
            "windows" => patterns.extend(app_config.include_windows.clone()),
            _ => {}
        }

        // Check whether any of the patterns match a file on disk
        let present = patterns.iter().any(|p| {
            let expanded = p.replace('~', &home.to_string_lossy());
            glob::glob(&expanded)
                .ok()
                .and_then(|mut m| m.next())
                .is_some()
        });

        if present {
            detected.push((preset_name.clone(), app_config));
        }
    }

    // ── 4. Report results ─────────────────────────────────────────────────
    if already_configured > 0 {
        println!("  ({} preset(s) already configured — skipped)", already_configured);
    }

    if detected.is_empty() {
        println!("\nNo new preset apps detected on this machine.");
        return Ok(());
    }

    println!("\nDetected {} app(s) installed on this machine:", detected.len());
    for (name, _) in &detected {
        println!("  - {}", name);
    }

    // ── 5. Prompt and batch-commit ────────────────────────────────────────
    let mut added: Vec<String> = Vec::new();

    for (name, app_config) in detected {
        print!("\nAdd '{}' to drifters? [y/N]: ", name);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("y")
            || input.trim().eq_ignore_ascii_case("yes")
        {
            rules.apps.insert(name.clone(), app_config);
            added.push(name);
        }
    }

    if added.is_empty() {
        println!("\nNo apps added.");
        return Ok(());
    }

    rules.save(repo_path)?;
    let commit_msg = format!("discover-presets: add {}", added.join(", "));
    commit_and_push(repo_path, &commit_msg)?;

    println!("\n✓ Added: {}", added.join(", "));
    println!("Run 'drifters push-app' to sync your new configs.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_repo() {
        // This test verifies that the Cargo.toml repository URL is valid
        let result = parse_github_repo();
        assert!(result.is_ok(), "Failed to parse repository URL from Cargo.toml");

        let (owner, repo) = result.unwrap();
        assert!(!owner.is_empty(), "Owner should not be empty");
        assert!(!repo.is_empty(), "Repo should not be empty");

        // For this project, it should be tjirsch/drifters
        assert_eq!(owner, "tjirsch");
        assert_eq!(repo, "drifters");
    }
}
