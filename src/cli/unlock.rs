use crate::config::LocalConfig;
use crate::error::Result;
use crate::git::confirm_operation;

pub fn unlock() -> Result<()> {
    let temp_repo = LocalConfig::get_temp_repo_path()?;
    let lock_path = temp_repo.with_extension("lock");

    if !lock_path.exists() {
        println!("No lock file found. Nothing to unlock.");
        return Ok(());
    }

    // Show info about the existing lock
    println!("Lock file: {:?}", lock_path);

    if let Ok(pid) = std::fs::read_to_string(&lock_path) {
        let pid = pid.trim();
        if !pid.is_empty() {
            println!("Held by PID: {}", pid);
        }
    }

    if let Ok(meta) = std::fs::metadata(&lock_path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = modified.elapsed() {
                println!("Age: {} seconds", age.as_secs());
            }
        }
    }

    println!();
    println!("Only remove this if drifters crashed or was interrupted with Ctrl-C.");

    if !confirm_operation("Remove lock file?", false)? {
        println!("Cancelled.");
        return Ok(());
    }

    std::fs::remove_file(&lock_path)?;
    println!("✓ Lock file removed");

    if temp_repo.exists() {
        println!("Cleaning up leftover temporary repository...");
        std::fs::remove_dir_all(&temp_repo)?;
        println!("✓ Temporary repository removed");
    }

    Ok(())
}
