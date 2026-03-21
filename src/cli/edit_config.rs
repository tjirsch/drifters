use crate::cli::common::open_file;
use crate::config::LocalConfig;
use crate::error::Result;

pub fn edit_config() -> Result<()> {
    let config = LocalConfig::load()?;
    let config_path = LocalConfig::config_file_path()?;

    println!("Opening {}...", config_path.display());
    open_file(&config_path, config.editor.as_deref())?;

    Ok(())
}
