use crate::error::{DriftersError, Result};
use std::path::PathBuf;

/// Generate (and optionally install) shell completion scripts.
///
/// `shell_str` – one of "bash", "zsh", "fish", "powershell".
/// `install`   – when true, write the script to the shell's default location
///               and print setup instructions; otherwise write to stdout.
pub fn run_completion(shell_str: &str, install: bool) -> Result<()> {
    use clap::CommandFactory;
    use clap_complete::{generate, Shell};
    use std::str::FromStr;

    let shell = Shell::from_str(shell_str).map_err(|_| {
        DriftersError::Config(format!(
            "Unknown shell '{}'. Supported shells: bash, zsh, fish, powershell",
            shell_str
        ))
    })?;

    let mut cmd = crate::Cli::command();
    let bin_name = "drifters";

    if install {
        let (path, post_install_msg) = completion_install_path(shell)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(&path)?;
        generate(shell, &mut cmd, bin_name, &mut file);
        println!("Completion script installed to: {}", path.display());
        if let Some(msg) = post_install_msg {
            println!("{}", msg);
        }
    } else {
        generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
    }

    Ok(())
}

fn completion_install_path(shell: clap_complete::Shell) -> Result<(PathBuf, Option<String>)> {
    use clap_complete::Shell;
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let (path, msg): (PathBuf, Option<String>) = match shell {
        Shell::Bash => (
            PathBuf::from(format!(
                "{}/.local/share/bash-completion/completions/drifters",
                home
            )),
            Some(
                "Ensure bash-completion is installed and sourced in your ~/.bashrc".to_string(),
            ),
        ),
        Shell::Zsh => (
            PathBuf::from(format!("{}/.zsh/completions/_drifters", home)),
            Some(
                "Ensure ~/.zsh/completions is in your fpath — add to ~/.zshrc:\n\
                   fpath=(~/.zsh/completions $fpath)\n\
                   autoload -Uz compinit && compinit"
                    .to_string(),
            ),
        ),
        Shell::Fish => (
            PathBuf::from(format!(
                "{}/.config/fish/completions/drifters.fish",
                home
            )),
            None,
        ),
        Shell::PowerShell => {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| home.clone());
            (
                PathBuf::from(format!(
                    r"{}\Documents\PowerShell\Completions\drifters.ps1",
                    userprofile
                )),
                Some(
                    "Add to your $PROFILE:\n\
                       . \"$env:USERPROFILE\\Documents\\PowerShell\\Completions\\drifters.ps1\""
                        .to_string(),
                ),
            )
        }
        _ => {
            return Err(DriftersError::Config(format!(
                "Unsupported shell: {:?}",
                shell
            )))
        }
    };
    Ok((path, msg))
}
