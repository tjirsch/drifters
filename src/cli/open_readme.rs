use crate::error::{DriftersError, Result};

const README_URL: &str =
    "https://raw.githubusercontent.com/tjirsch/drifters/main/README.md";

/// Download the latest README from the repository and open it with the
/// preferred editor (or the OS default if none is configured).
pub fn run_open_readme(preferred_editor: Option<&str>) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("drifters-open-readme")
        .build()?;

    println!("📄 Downloading README...");
    let response = client.get(README_URL).send()?;
    if !response.status().is_success() {
        return Err(DriftersError::Config(format!(
            "Failed to download README: HTTP {}",
            response.status()
        )));
    }
    let content = response.bytes()?;

    // Save to ~/Downloads/drifters-README.md, falling back to the system temp dir
    let dest = {
        let downloads = dirs::download_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(std::env::temp_dir);
        downloads.join("drifters-README.md")
    };

    std::fs::write(&dest, &content)?;
    println!("README saved to: {}", dest.display());

    crate::cli::common::open_file(&dest, preferred_editor)?;
    Ok(())
}
