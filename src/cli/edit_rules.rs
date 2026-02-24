use crate::cli::common::open_file;
use crate::config::LocalConfig;
use crate::error::Result;
use crate::git::{commit_and_push, confirm_operation, EphemeralRepoGuard};

pub fn edit_rules() -> Result<()> {
    let local_config = LocalConfig::load()?;
    let repo_guard = EphemeralRepoGuard::new(&local_config)?;
    let repo_path = repo_guard.path();

    let rules_path = repo_path.join(".drifters").join("sync-rules.toml");

    if !rules_path.exists() {
        return Err(crate::error::DriftersError::Config(
            "sync-rules.toml not found in repository".to_string(),
        ));
    }

    println!("Opening sync-rules.toml...");
    println!("(The repository lock is held while the editor is open)");

    open_file(&rules_path, local_config.preferred_editor.as_deref())?;

    // For GUI editors that return immediately, give the user a chance to finish editing.
    {
        use std::io::{self, Write};
        print!("Press Enter when you have finished editing...");
        io::stdout().flush()?;
        let mut _buf = String::new();
        io::stdin().read_line(&mut _buf)?;
    }

    if confirm_operation("Save changes to repository?", false)? {
        commit_and_push(
            repo_path,
            &format!("Edit sync rules from {}", local_config.machine_id),
        )?;
        println!("✓ Changes saved to repository");
    } else {
        println!("Changes discarded");
    }

    Ok(())
}
