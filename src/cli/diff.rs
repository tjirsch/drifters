use crate::config::{resolve_fileset, LocalConfig, SyncRules};
use crate::error::Result;
use crate::git::{read_app_files, EphemeralRepoGuard};
use std::fs;

pub fn show_diff(app_name: Option<String>, against: Option<String>) -> Result<()> {
    log::info!("Showing diff");

    // Load local config
    let config = LocalConfig::load()?;

    // Determine comparison branch
    let compare_branch = against.unwrap_or_else(|| "main".to_string());

    // Set up ephemeral repo on the comparison branch
    println!("Fetching latest from repository...");
    let repo_guard = EphemeralRepoGuard::new_on_branch(&config, &compare_branch)?;
    let repo_path = repo_guard.path();

    // Guard: detect stale machine IDs
    crate::cli::common::verify_machine_registration(&config, repo_path)?;

    // Load sync rules from main
    let rules = load_rules_from_main(repo_path)?;

    if rules.apps.is_empty() {
        println!("No apps configured for sync.");
        return Ok(());
    }

    // Determine which apps to diff
    let apps_to_diff: Vec<_> = if let Some(name) = app_name {
        if rules.apps.contains_key(&name) {
            vec![name]
        } else {
            return Err(crate::error::DriftersError::AppNotFound(name));
        }
    } else {
        rules.apps.keys().cloned().collect()
    };

    println!("Comparing local files against branch '{}'", compare_branch);

    let mut total_changes = 0;

    for app in &apps_to_diff {
        let app_config = rules.apps.get(app).unwrap();

        println!("\n{}", "=".repeat(60));
        println!("App: {}", app);
        println!("{}", "=".repeat(60));

        // Resolve fileset for THIS machine
        let fileset = resolve_fileset(
            app_config,
            &config.machine_id,
            std::env::consts::OS,
        )?;

        if fileset.is_empty() {
            println!("  (no files in fileset for this machine)");
            continue;
        }

        // Read files from the comparison branch
        let remote_files = read_app_files(repo_path, app)?;

        for local_path in fileset {
            let filename = local_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let remote_content = match remote_files.get(filename) {
                Some(content) => content.clone(),
                None => continue,
            };

            // Compare with local
            let local_content = if local_path.exists() {
                fs::read_to_string(&local_path)?
            } else {
                String::new()
            };

            // Show diff if different
            if local_content != remote_content {
                println!("\n{} ({})", filename, local_path.display());
                println!("{}", "-".repeat(60));
                show_file_diff(&local_content, &remote_content);
                total_changes += 1;
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    if total_changes == 0 {
        println!("All configs are up to date with '{}'", compare_branch);
    } else {
        println!("{} file(s) differ from '{}'", total_changes, compare_branch);
        println!("\nRun 'drifters pull-app' to apply changes from main");
    }

    Ok(())
}

fn load_rules_from_main(repo_path: &std::path::Path) -> Result<SyncRules> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["show", "main:.drifters/sync-rules.toml"])
        .output()?;

    if !output.status.success() {
        return SyncRules::load(&repo_path.to_path_buf());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let rules: SyncRules = toml::from_str(&content)?;
    Ok(rules)
}

fn show_file_diff(old: &str, new: &str) {
    use similar::TextDiff;

    let diff = TextDiff::from_lines(old, new);
    let mut shown_lines = 0;
    const MAX_LINES: usize = 100;

    for change in diff.iter_all_changes() {
        if shown_lines >= MAX_LINES {
            println!("  ... (more changes, {} lines total)", diff.iter_all_changes().count());
            break;
        }

        match change.tag() {
            similar::ChangeTag::Delete => {
                print!("  \x1b[31m-{}\x1b[0m", change);
                shown_lines += 1;
            }
            similar::ChangeTag::Insert => {
                print!("  \x1b[32m+{}\x1b[0m", change);
                shown_lines += 1;
            }
            similar::ChangeTag::Equal => {
                if shown_lines > 0 && shown_lines < MAX_LINES - 3 {
                    print!("   {}", change);
                }
            }
        }
    }
}
