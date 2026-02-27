mod cli;
mod config;
mod error;
mod git;
mod merge;
mod parser;
mod sync;

use clap::{Parser, Subcommand};
use error::Result;

#[derive(Parser)]
#[command(name = "drifters")]
#[command(version)]
#[command(about = "Config file synchronization across machines", long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(long, global = true)]
    yolo: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize drifters on this machine
    #[command(arg_required_else_help = true)]
    Init {
        /// GitHub repository URL
        repo_url: String,
    },
    /// Add an app to sync
    #[command(arg_required_else_help = true)]
    AddApp {
        /// App name to add
        app_name: String,
    },
    /// Push local configs to repository
    PushApp {
        /// Optional app name to push (all if not specified)
        app_name: Option<String>,
    },
    /// Pull configs from repository
    PullApp {
        /// Optional app name to pull (all if not specified)
        app_name: Option<String>,
    },
    /// List all apps configured for sync (detailed)
    ListApp {
        /// Optional app name to show details for
        app_name: Option<String>,
    },
    /// Print current sync-rules.toml
    ListRules,
    /// Remove an app's configs from this machine, a specific machine, or all machines
    #[command(arg_required_else_help = true)]
    RemoveApp {
        /// App name to remove
        app_name: String,
        /// Remove from this specific machine ID instead of the local machine
        #[arg(long)]
        machine: Option<String>,
        /// Remove from ALL machines and delete the app from sync-rules entirely
        #[arg(long)]
        all: bool,
    },
    /// Rename an app in the registry and repo
    #[command(arg_required_else_help = true)]
    RenameApp {
        /// Current app name
        old_name: String,
        /// New app name
        new_name: String,
    },
    /// Exclude a file from syncing on this machine
    #[command(arg_required_else_help = true)]
    ExcludeApp {
        /// App name
        app_name: String,
        /// Filename to exclude (e.g., "settings.json")
        filename: String,
    },
    /// Show sync status
    Status,
    /// Show diff without applying changes
    DiffApp {
        /// Optional app name to diff
        app_name: Option<String>,
    },
    /// Re-merge configs using current rules
    MergeApp {
        /// Optional app name to merge
        app_name: Option<String>,

        /// Only consider state from specific machine
        #[arg(long)]
        machine: Option<String>,

        /// Use OS-specific rules for this OS
        #[arg(long)]
        os: Option<String>,

        /// Show what would change without applying
        #[arg(long)]
        dry_run: bool,
    },
    /// Import app definition from file (defaults to ./<app>.toml)
    #[command(arg_required_else_help = true)]
    ImportApp {
        /// App name
        app_name: String,
        /// File to import from (optional, defaults to ./<app>.toml)
        #[arg(long)]
        file: Option<std::path::PathBuf>,
    },
    /// Export app definition to file (defaults to ./<app>.toml)
    #[command(arg_required_else_help = true)]
    ExportApp {
        /// App name
        app_name: String,
        /// File to export to (optional, defaults to ./<app>.toml)
        #[arg(long)]
        file: Option<std::path::PathBuf>,
    },
    /// Import entire sync-rules.toml from file (defaults to ./sync-rules.toml)
    ImportRules {
        /// File to import from (optional, defaults to ./sync-rules.toml)
        #[arg(long)]
        file: Option<std::path::PathBuf>,
    },
    /// Export entire sync-rules.toml to file (defaults to ./sync-rules.toml)
    ExportRules {
        /// File to export to (optional, defaults to ./sync-rules.toml)
        #[arg(long)]
        file: Option<std::path::PathBuf>,
    },
    /// List available presets from GitHub repository
    ListPresets,
    /// Load preset from GitHub repository
    #[command(arg_required_else_help = true)]
    LoadPreset {
        /// Preset name (e.g., "zed", "vscode")
        preset_name: String,
    },
    /// Auto-detect installed apps on this machine and offer to add them from presets
    DiscoverPresets,
    /// Show history of rules or app
    #[command(arg_required_else_help = true)]
    History {
        #[command(subcommand)]
        target: HistoryTarget,
    },
    /// Restore previous version of rules or app
    #[command(arg_required_else_help = true)]
    Restore {
        #[command(subcommand)]
        target: RestoreTarget,
    },
    /// Rename a machine in the registry and repo
    #[command(arg_required_else_help = true)]
    RenameMachine {
        /// Current machine ID
        old_id: String,
        /// New machine ID
        new_id: String,
    },
    /// Remove a machine from the registry and delete its configs
    #[command(arg_required_else_help = true)]
    RemoveMachine {
        /// Machine ID to remove
        machine_id: String,
    },
    /// Generate shell hook for auto-pull
    Hook,
    /// Check for and install new releases from GitHub
    SelfUpdate {
        /// Only check if an update is available; do not install
        #[arg(long)]
        check_only: bool,
        /// Skip SHA-256 checksum verification (not recommended; use only for
        /// releases that predate checksum support)
        #[arg(long)]
        skip_checksum: bool,
        /// Do not download README.md after installing an update
        #[arg(long)]
        no_download_readme: bool,
        /// Do not open README.md after downloading (only applies if download runs)
        #[arg(long)]
        no_open_readme: bool,
    },
    /// Download and open the latest README from the repository
    OpenReadme,
    /// Generate shell completion script
    Completion {
        /// Shell to generate completions for: bash, zsh, fish, powershell
        /// (defaults to zsh on macOS)
        shell: Option<String>,
        /// Install the completion script to the default location for the shell
        /// (default on macOS when no shell is specified)
        #[arg(long)]
        install: bool,
    },
    /// Set (or clear) the preferred editor in local config
    SetEditor {
        /// Editor command to use (e.g. "code", "zed", "vim"). Omit to show current value.
        editor: Option<String>,
        /// Remove the editor setting (fall back to $EDITOR / OS default)
        #[arg(long)]
        clear: bool,
    },
    /// Open sync-rules.toml in your editor and optionally save changes to the repository
    EditRules,
    /// Force-remove a stale lock file left behind after a crash or Ctrl-C
    Unlock,
}

#[derive(Subcommand)]
enum HistoryTarget {
    /// Show history of all rules
    Rules {
        /// Number of commits to show
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Show diff for specific commit
        #[arg(long)]
        commit: Option<String>,
    },
    /// Show history of specific app
    App {
        /// App name
        app_name: String,
        /// Number of commits to show
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Show diff for specific commit
        #[arg(long)]
        commit: Option<String>,
    },
}

#[derive(Subcommand)]
enum RestoreTarget {
    /// Restore app from previous commit
    App {
        /// App name
        app_name: String,
        /// Commit hash to restore from
        #[arg(long)]
        commit: String,
    },
    /// Restore entire rules from previous commit
    Rules {
        /// Commit hash to restore from
        #[arg(long)]
        commit: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logger
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    // Check for updates (unless running self-update, init, or machine management commands)
    if !matches!(
        cli.command,
        Commands::SelfUpdate { .. }
            | Commands::Init { .. }
            | Commands::RenameMachine { .. }
            | Commands::RemoveMachine { .. }
            | Commands::OpenReadme
            | Commands::Completion { .. }
            | Commands::SetEditor { .. }
            | Commands::Unlock
    ) {
        if let Ok(mut config) = config::LocalConfig::load() {
            let _ = cli::self_update::maybe_check_for_updates(&mut config);
        }
    }

    match cli.command {
        Commands::Init { repo_url } => {
            cli::init::initialize(repo_url)
        }
        Commands::AddApp { app_name } => {
            cli::add::add_app(app_name)
        }
        Commands::PushApp { app_name } => {
            cli::push::push_command(app_name, cli.yolo)
        }
        Commands::PullApp { app_name } => {
            cli::pull::pull_command(app_name, cli.yolo)
        }
        Commands::ListApp { app_name } => {
            cli::list::list_apps(app_name)
        }
        Commands::ListRules => {
            cli::list::list_rules()
        }
        Commands::RemoveApp { app_name, machine, all } => {
            cli::remove::remove_app(app_name, machine, all)
        }
        Commands::RenameApp { old_name, new_name } => {
            cli::rename_app::rename_app(old_name, new_name)
        }
        Commands::ExcludeApp { app_name, filename } => {
            cli::exclude::exclude_file(app_name, filename)
        }
        Commands::Status => {
            cli::status::show_status()
        }
        Commands::DiffApp { app_name } => {
            cli::diff::show_diff(app_name)
        }
        Commands::MergeApp { app_name, machine, os, dry_run } => {
            cli::merge::merge_command(app_name, machine, os, dry_run, cli.yolo)
        }
        Commands::ImportApp { app_name, file } => {
            cli::import::import_app(app_name, file)
        }
        Commands::ExportApp { app_name, file } => {
            cli::export::export_app(app_name, file)
        }
        Commands::ImportRules { file } => {
            cli::import::import_rules(file)
        }
        Commands::ExportRules { file } => {
            cli::export::export_rules(file)
        }
        Commands::ListPresets => {
            cli::presets::list_presets()
        }
        Commands::LoadPreset { preset_name } => {
            cli::presets::load_preset(preset_name)
        }
        Commands::DiscoverPresets => {
            cli::presets::discover_presets()
        }
        Commands::History { target } => match target {
            HistoryTarget::Rules { limit, commit } => {
                if let Some(hash) = commit {
                    cli::history::show_commit_diff(hash, None)
                } else {
                    cli::history::show_history_rules(limit)
                }
            }
            HistoryTarget::App { app_name, limit, commit } => {
                if let Some(hash) = commit {
                    cli::history::show_commit_diff(hash, Some(app_name))
                } else {
                    cli::history::show_history_app(app_name, limit)
                }
            }
        }
        Commands::Restore { target } => match target {
            RestoreTarget::App { app_name, commit } => {
                cli::restore::restore_app(app_name, commit)
            }
            RestoreTarget::Rules { commit } => {
                cli::restore::restore_rules(commit)
            }
        }
        Commands::RenameMachine { old_id, new_id } => {
            cli::rename_machine::rename_machine(old_id, new_id)
        }
        Commands::RemoveMachine { machine_id } => {
            cli::remove_machine::remove_machine(machine_id)
        }
        Commands::Hook => {
            cli::hook::generate_hook()
        }
        Commands::SelfUpdate { check_only, skip_checksum, no_download_readme, no_open_readme } => {
            let editor = config::LocalConfig::load()
                .ok()
                .and_then(|c| c.editor);
            cli::self_update::run_self_update(
                check_only,
                skip_checksum,
                no_download_readme,
                no_open_readme,
                editor.as_deref(),
            )
        }
        Commands::OpenReadme => {
            let editor = config::LocalConfig::load()
                .ok()
                .and_then(|c| c.editor);
            cli::open_readme::run_open_readme(editor.as_deref())
        }
        Commands::Completion { shell, install } => {
            cli::completion::run_completion(shell.as_deref(), install)
        }
        Commands::SetEditor { editor, clear } => {
            let mut config = config::LocalConfig::load()?;
            if clear {
                config.editor = None;
                config.save()?;
                println!("✅ editor cleared (will fall back to $EDITOR / OS default).");
            } else if let Some(e) = editor {
                config.editor = Some(e.clone());
                config.save()?;
                println!("✅ editor set to \"{}\".", e);
            } else {
                match &config.editor {
                    Some(e) => println!("editor = \"{}\"", e),
                    None => println!("editor is not set (using $EDITOR / OS default)."),
                }
            }
            Ok(())
        }
        Commands::EditRules => {
            cli::edit_rules::edit_rules()
        }
        Commands::Unlock => {
            cli::unlock::unlock()
        }
    }
}
