use crate::config::LocalConfig;
use crate::error::{DriftersError, Result};
use crate::git::{clone_repo, pull_latest};
use std::path::PathBuf;

// ─── Lock constants ──────────────────────────────────────────────────────────
/// Maximum time (seconds) to wait for another process to release the lock.
const LOCK_TIMEOUT_SECS: u64 = 30;
/// How long (seconds) a lock file must be before we treat it as stale
/// (i.e. the owning process died without cleaning up).
const LOCK_STALE_SECS: u64 = 300; // 5 minutes

// ─── Ephemeral repo helpers ──────────────────────────────────────────────────

/// Set up ephemeral repo for this command.
/// Clones if it doesn't exist, pulls if it does.
pub fn setup_ephemeral_repo(config: &LocalConfig) -> Result<PathBuf> {
    let temp_repo = LocalConfig::get_temp_repo_path()?;

    if temp_repo.exists() {
        log::debug!("Temp repo exists, pulling latest");
        pull_latest(&temp_repo)?;
    } else {
        log::debug!("Cloning repo to temp location");
        clone_repo(&config.repo_url, &temp_repo)?;
    }

    Ok(temp_repo)
}

/// Clean up ephemeral repo after command completes.
pub fn cleanup_ephemeral_repo() -> Result<()> {
    let temp_repo = LocalConfig::get_temp_repo_path()?;

    if temp_repo.exists() {
        log::debug!("Cleaning up temp repo at {:?}", temp_repo);
        std::fs::remove_dir_all(&temp_repo)?;
    }

    Ok(())
}

// ─── Lock file helpers ───────────────────────────────────────────────────────

fn lock_path() -> Result<PathBuf> {
    let temp_repo = LocalConfig::get_temp_repo_path()?;
    // Sibling file: ~/.config/drifters/tmp-repo.lock
    Ok(temp_repo.with_extension("lock"))
}

/// Try to atomically create the lock file with the current PID.
/// Returns `true` on success, `false` if the file already exists
/// (and is not stale).
fn try_acquire_lock(path: &PathBuf) -> Result<bool> {
    use std::fs::OpenOptions;
    use std::io::Write;

    // `create_new` is atomic on POSIX: succeeds only if the file does not exist.
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut f) => {
            // Write PID so stale-lock detection can check if the owner is alive
            let _ = write!(f, "{}", std::process::id());
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Lock exists — check if it's stale
            if is_stale_lock(path) {
                log::warn!("Removing stale lock file at {:?}", path);
                let _ = std::fs::remove_file(path);
                // Try once more
                match OpenOptions::new().write(true).create_new(true).open(path) {
                    Ok(mut f) => {
                        let _ = write!(f, "{}", std::process::id());
                        Ok(true)
                    }
                    Err(_) => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
        Err(e) => Err(DriftersError::Io(e)),
    }
}

/// Returns true if the lock file is older than `LOCK_STALE_SECS`.
fn is_stale_lock(path: &PathBuf) -> bool {
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = modified.elapsed() {
                return age.as_secs() > LOCK_STALE_SECS;
            }
        }
    }
    false
}

/// Acquire the lock, spinning up to `LOCK_TIMEOUT_SECS`.
fn acquire_lock(path: &PathBuf) -> Result<()> {
    let start = std::time::Instant::now();
    let mut printed_waiting = false;

    loop {
        if try_acquire_lock(path)? {
            return Ok(());
        }

        if start.elapsed().as_secs() >= LOCK_TIMEOUT_SECS {
            return Err(DriftersError::Config(format!(
                "Timed out waiting for another drifters process to finish \
                 (lock file: {:?}). If no other process is running, delete \
                 the lock file manually.",
                path
            )));
        }

        if !printed_waiting {
            println!("⏳ Another drifters process is running; waiting...");
            printed_waiting = true;
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

/// Release the lock by removing the lock file.
fn release_lock(path: &PathBuf) {
    if let Err(e) = std::fs::remove_file(path) {
        // Not fatal — next invocation will detect the stale lock
        log::warn!("Failed to remove lock file {:?}: {}", path, e);
    }
}

// ─── RAII guard ──────────────────────────────────────────────────────────────

/// RAII guard that:
/// 1. Acquires a lock file before touching the shared temp repo.
/// 2. Sets up (clones or pulls) the ephemeral repo.
/// 3. Releases the lock and cleans up the repo on `Drop`.
///
/// Prevents two concurrent drifters processes from corrupting the shared
/// temp repo at `~/.config/drifters/tmp-repo`.
pub struct EphemeralRepoGuard {
    repo_path: PathBuf,
    lock_path: PathBuf,
}

impl EphemeralRepoGuard {
    pub fn new(config: &LocalConfig) -> Result<Self> {
        let lock_path = lock_path()?;

        // Acquire the lock first — blocks if another process holds it
        acquire_lock(&lock_path)?;

        // Set up the repo (may fail; Drop will still release the lock)
        match setup_ephemeral_repo(config) {
            Ok(repo_path) => Ok(Self {
                repo_path,
                lock_path,
            }),
            Err(e) => {
                release_lock(&lock_path);
                Err(e)
            }
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.repo_path
    }
}

impl Drop for EphemeralRepoGuard {
    fn drop(&mut self) {
        if let Err(e) = cleanup_ephemeral_repo() {
            log::warn!("Failed to cleanup ephemeral repo: {}", e);
        }
        release_lock(&self.lock_path);
    }
}
