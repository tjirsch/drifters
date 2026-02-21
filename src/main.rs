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
#[command(about = "Config file synchronization across machines", long_about = None)]
struct Cli {
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
    Init {
        /// GitHub repository URL
        repo_url: String,
    },
    /// Add an app to sync
    Add {
        /// App name to add
        app_name: String,
    },
    /// Push local configs to repository
    Push {
        /// Optional app name to push (all if not specified)
        app_name: Option<String>,
    },
    /// Pull configs from repository
    Pull {
        /// Optional app name to pull (all if not specified)
        app_name: Option<String>,
    },
    /// List all apps configured for sync
    List,
    /// Exclude a file from syncing on this machine
    Exclude {
        /// App name
        app_name: String,
        /// Filename to exclude (e.g., "settings.json")
        filename: String,
    },
    /// Show sync status
    Status,
    /// Show diff without applying changes
    Diff {
        /// Optional app name to diff
        app_name: Option<String>,
    },
    /// Re-merge configs using current rules
    Merge {
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
    /// Import app or rules from file
    Import {
        #[command(subcommand)]
        target: ImportTarget,
    },
    /// Export app or rules to file
    Export {
        #[command(subcommand)]
        target: ExportTarget,
    },
    /// Show history of rules or app
    History {
        #[command(subcommand)]
        target: HistoryTarget,
    },
    /// Restore previous version of rules or app
    Restore {
        #[command(subcommand)]
        target: RestoreTarget,
    },
    /// Generate shell hook for auto-pull
    Hook,
    /// Check for and install new releases from GitHub
    SelfUpdate {
        /// Only check if an update is available; do not install
        #[arg(long)]
        check_only: bool,
    },
}

#[derive(Subcommand)]
enum ImportTarget {
    /// Import app definition from file
    App {
        /// App name
        app_name: String,
        /// File to import from
        #[arg(long)]
        file: std::path::PathBuf,
    },
    /// Import entire rules file
    Rules {
        /// File to import from
        #[arg(long)]
        file: std::path::PathBuf,
    },
}

#[derive(Subcommand)]
enum ExportTarget {
    /// Export app definition to file
    App {
        /// App name
        app_name: String,
        /// File to export to
        #[arg(long)]
        file: std::path::PathBuf,
    },
    /// Export entire rules file
    Rules {
        /// File to export to
        #[arg(long)]
        file: std::path::PathBuf,
    },
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

fn main() -> Result<()> {
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

    // Check for updates (unless running self-update or init command)
    if !matches!(cli.command, Commands::SelfUpdate { .. } | Commands::Init { .. }) {
        if let Ok(mut config) = config::LocalConfig::load() {
            let _ = cli::self_update::maybe_check_for_updates(&mut config);
        }
    }

    match cli.command {
        Commands::Init { repo_url } => {
            cli::init::initialize(repo_url)
        }
        Commands::Add { app_name } => {
            cli::add::add_app(app_name)
        }
        Commands::Push { app_name } => {
            cli::push::push_command(app_name, cli.yolo)
        }
        Commands::Pull { app_name } => {
            cli::pull::pull_command(app_name, cli.yolo)
        }
        Commands::List => {
            cli::list::list_apps()
        }
        Commands::Exclude { app_name, filename } => {
            cli::exclude::exclude_file(app_name, filename)
        }
        Commands::Status => {
            cli::status::show_status()
        }
        Commands::Diff { app_name } => {
            cli::diff::show_diff(app_name)
        }
        Commands::Merge { app_name, machine, os, dry_run } => {
            cli::merge::merge_command(app_name, machine, os, dry_run, cli.yolo)
        }
        Commands::Import { target } => match target {
            ImportTarget::App { app_name, file } => {
                cli::import::import_app(app_name, file)
            }
            ImportTarget::Rules { file } => {
                cli::import::import_rules(file)
            }
        }
        Commands::Export { target } => match target {
            ExportTarget::App { app_name, file } => {
                cli::export::export_app(app_name, file)
            }
            ExportTarget::Rules { file } => {
                cli::export::export_rules(file)
            }
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
        Commands::Hook => {
            cli::hook::generate_hook()
        }
        Commands::SelfUpdate { check_only } => {
            cli::self_update::run_self_update(check_only)
        }
    }
}
