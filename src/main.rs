mod cli;
mod config;
mod error;
mod git;
mod merge;
mod parser;
mod sync;
mod tui;

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
    /// Generate shell hook for auto-pull
    Hook,
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
            println!("Showing diff{}",
                app_name.map(|a| format!(" for {}", a)).unwrap_or_default());
            // TODO: Implement diff
            Ok(())
        }
        Commands::Hook => {
            cli::hook::generate_hook()
        }
    }
}
