use crate::config::{LocalConfig, MachineRegistry};
use crate::error::{DriftersError, Result};
use std::io::{self, Write};

/// Verify that the local machine ID is still registered in the shared repo.
///
/// This guards against the case where another machine runs `rename-machine` or
/// `remove-machine` while this machine is offline — leaving this machine's
/// `~/.config/drifters/config.toml` holding a stale ID.
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
