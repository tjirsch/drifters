use crate::config::{LocalConfig, SyncRules};
use crate::error::{DriftersError, Result};
use crate::git::{commit_and_push, confirm_operation, EphemeralRepoGuard};

/// Rename an app in sync-rules and in the repo directory structure.
///
/// Renames:
///   • `apps/<old-name>/` → `apps/<new-name>/`  (all machine configs)
///   • The app key in `.drifters/sync-rules.toml`
///
/// This affects all machines — they will see the new name on their next
/// `drifters push-app` or `drifters pull-app`.
pub fn rename_app(old_name: String, new_name: String) -> Result<()> {
    log::info!("Renaming app '{}' → '{}'", old_name, new_name);

    // ── Validate new_name before touching anything ────────────────────────────
    if new_name.is_empty() {
        return Err(DriftersError::Config(
            "New app name cannot be empty.".to_string(),
        ));
    }
    if new_name.contains('/') || new_name.contains('\\') {
        return Err(DriftersError::Config(
            "New app name cannot contain '/' or '\\'.".to_string(),
        ));
    }
    if new_name == old_name {
        return Err(DriftersError::Config(format!(
            "New app name is the same as the current one ('{}').",
            old_name
        )));
    }

    // ── Load config and set up ephemeral repo ─────────────────────────────────
    let config = LocalConfig::load()?;

    println!("Fetching latest repository...");
    let repo_guard = EphemeralRepoGuard::new(&config)?;
    let repo_path = repo_guard.path();

    // ── Load sync rules ───────────────────────────────────────────────────────
    let mut rules = SyncRules::load(repo_path)?;

    // ── Validate old_name exists ──────────────────────────────────────────────
    if !rules.apps.contains_key(&old_name) {
        return Err(DriftersError::AppNotFound(old_name));
    }

    // ── Validate new_name is not already taken ────────────────────────────────
    if rules.apps.contains_key(&new_name) {
        return Err(DriftersError::Config(format!(
            "App '{}' already exists. Choose a different name.",
            new_name
        )));
    }

    // ── Confirm with user ─────────────────────────────────────────────────────
    println!("\nRename app '{}' → '{}'", old_name, new_name);
    println!("This will:");
    println!(
        "  • Rename apps/{old}/ → apps/{new}/ in the repo",
        old = old_name,
        new = new_name
    );
    println!("  • Update the app entry in sync-rules.toml");
    println!("  Note: This affects all machines — they will see the new name on next sync.");

    if !confirm_operation("Proceed with rename?", false)? {
        println!("Cancelled.");
        return Ok(());
    }

    // ── Rename the app key in SyncRules ───────────────────────────────────────
    // Persist the TOML change BEFORE renaming the directory so that if save()
    // fails the on-disk layout is still intact and the repo is left clean.
    let app_config = rules.apps.remove(&old_name).ok_or_else(|| {
        DriftersError::Config(format!(
            "App '{}' disappeared from rules during rename",
            old_name
        ))
    })?;
    rules.apps.insert(new_name.clone(), app_config);

    // ── Persist changes ───────────────────────────────────────────────────────
    let repo_path_buf = repo_path.to_path_buf();
    rules.save(&repo_path_buf)?;

    // ── Rename apps/<old>/ → apps/<new>/ (after TOML is safely persisted) ────
    let old_app_dir = repo_path.join("apps").join(&old_name);
    let new_app_dir = repo_path.join("apps").join(&new_name);

    let dir_renamed = if old_app_dir.exists() {
        std::fs::rename(&old_app_dir, &new_app_dir)?;
        log::debug!("Renamed {:?} → {:?}", old_app_dir, new_app_dir);
        true
    } else {
        false
    };

    // ── Commit and push ───────────────────────────────────────────────────────
    commit_and_push(
        &repo_path_buf,
        &format!("rename app '{}' to '{}'", old_name, new_name),
    )?;

    println!("\n✓ App '{}' renamed to '{}'.", old_name, new_name);
    if dir_renamed {
        println!(
            "  Config directories moved: apps/{}/ → apps/{}/",
            old_name, new_name
        );
    }
    println!(
        "  Other machines will see the new name on their next 'drifters push-app' or 'drifters pull-app'."
    );

    Ok(())
}
