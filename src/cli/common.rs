use crate::config::{LocalConfig, MachineRegistry};
use crate::error::{DriftersError, Result};
use std::io::{self, Write};
use std::path::Path;

/// Open a file using `preferred_editor`, falling back to `$EDITOR`, then the OS default.
///
/// Priority:
/// 1. `preferred_editor` argument (from `LocalConfig.preferred_editor`)
/// 2. `$EDITOR` environment variable
/// 3. OS default: `open` on macOS, `xdg-open` on Linux, `cmd /C start` on Windows
///
/// On macOS, if the named editor binary is not found on `PATH`, falls back to
/// `open -a <editor> <file>` so GUI apps (Zed, VS Code, etc.) can be found by
/// their app-bundle name even when their CLI wrapper is absent.
pub fn open_file(path: &Path, preferred_editor: Option<&str>) -> Result<()> {
    let path_str = path.to_str().ok_or_else(|| {
        DriftersError::Config(format!(
            "File path {:?} contains non-UTF-8 characters",
            path
        ))
    })?;

    let editor_env = std::env::var("EDITOR").ok();
    let editor = preferred_editor.or_else(|| editor_env.as_deref());

    if let Some(editor) = editor {
        println!("   Opening '{}' with '{}'...", path_str, editor);
        let result = std::process::Command::new(editor).arg(path).status();
        match result {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                #[cfg(target_os = "macos")]
                {
                    let open_result = std::process::Command::new("open")
                        .args(["-a", editor, path_str])
                        .status();
                    if open_result.map(|s| s.success()).unwrap_or(false) {
                        return Ok(());
                    }
                }
                return Err(DriftersError::Config(format!(
                    "Editor '{}' not found — is it installed and on your PATH?\n\
                     Hint: set preferred_editor to the full path in ~/.config/drifters/drifters.toml\n\
                     e.g.  preferred_editor = \"/usr/local/bin/zed\"",
                    editor
                )));
            }
            Err(e) => {
                return Err(DriftersError::Config(format!(
                    "Failed to launch editor '{}': {}",
                    editor, e
                )))
            }
        }
    }

    // No editor configured — use OS default
    #[cfg(target_os = "macos")]
    {
        println!("   Opening '{}' with system default app...", path_str);
        std::process::Command::new("open")
            .arg(path_str)
            .status()
            .map_err(|e| {
                DriftersError::Config(format!("Failed to open '{}' with 'open': {}", path_str, e))
            })?;
    }
    #[cfg(target_os = "linux")]
    {
        println!("   Opening '{}' with xdg-open...", path_str);
        if std::process::Command::new("xdg-open")
            .arg(path_str)
            .status()
            .is_err()
        {
            return Err(DriftersError::Config(format!(
                "Could not open '{}': xdg-open failed and neither preferred_editor nor $EDITOR is set",
                path_str
            )));
        }
    }
    #[cfg(target_os = "windows")]
    {
        println!("   Opening '{}' with system default app...", path_str);
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path_str])
            .status()
            .map_err(|e| {
                DriftersError::Config(format!("Failed to open '{}': {}", path_str, e))
            })?;
    }
    Ok(())
}

/// Verify that the local machine ID is still registered in the shared repo.
///
/// This guards against the case where another machine runs `rename-machine` or
/// `remove-machine` while this machine is offline — leaving this machine's
/// `~/.config/drifters/drifters.toml` holding a stale ID.
///
/// Call this after `EphemeralRepoGuard::new()` in any command that depends on
/// the machine ID being valid (push, pull, status, diff, merge, …).
///
/// Returns `Ok(())` to let the caller proceed, or `Err(...)` if the user
/// chooses to exit.
pub fn verify_machine_registration(
    config: &LocalConfig,
    repo_path: &std::path::Path,
) -> Result<()> {
    let registry = MachineRegistry::load(&repo_path.to_path_buf())?;

    // Happy path — ID is registered, nothing to do
    if registry.machines.contains_key(&config.machine_id) {
        return Ok(());
    }

    // ── Stale ID detected ─────────────────────────────────────────────────────
    eprintln!(
        "\n⚠️  Your machine ID '{}' is no longer registered in this repo.",
        config.machine_id
    );
    eprintln!(
        "   It may have been renamed or removed from another machine."
    );

    let mut known: Vec<_> = registry.machines.keys().cloned().collect();
    known.sort();
    if known.is_empty() {
        eprintln!("   Registered machines: (none)");
    } else {
        eprintln!("   Registered machines: {}", known.join(", "));
    }

    eprintln!();
    eprintln!("What would you like to do?");
    eprintln!("  [1] Continue anyway (this machine will be treated as unregistered)");
    eprintln!("  [2] Exit — run 'drifters init <repo-url>' to re-initialize");

    loop {
        print!("Choice [1/2]: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" => {
                eprintln!("Continuing with unregistered machine ID '{}'.", config.machine_id);
                return Ok(());
            }
            "2" => {
                return Err(DriftersError::Config(format!(
                    "Machine '{}' is not registered. \
                     Run 'drifters init <repo-url>' to re-initialize this machine.",
                    config.machine_id
                )));
            }
            other => {
                eprintln!("  Please enter '1' or '2' (got '{}').", other);
            }
        }
    }
}

