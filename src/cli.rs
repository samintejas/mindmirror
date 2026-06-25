//! Command-line interface (clap), replacing the Go version's Cobra commands.
//!
//! The old `run` and `tray` commands are unified into `serve [--watch]`; a new
//! `source` command group manages content sources from the terminal (mirroring
//! the web manager), and `init` scaffolds config + assets.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::Config;
use crate::error::Result;

#[derive(Parser, Debug)]
#[command(
    name = "mindmirror",
    version,
    about = "A minimal static-site generator for markdown notes"
)]
pub struct Cli {
    /// Path to a config file (overrides discovery).
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build HTML for every registered source.
    Build,

    /// Serve the built site and source manager.
    Serve {
        /// Port to listen on (overrides config).
        #[arg(short, long)]
        port: Option<u16>,
        /// Build on start, watch local sources, and live-reload the browser.
        #[arg(long)]
        watch: bool,
    },

    /// Remove the build output directory.
    Clean,

    /// Write a default config file and materialize default assets.
    Init,

    /// Manage content sources.
    Source {
        #[command(subcommand)]
        action: SourceAction,
    },
}

#[derive(Subcommand, Debug)]
enum SourceAction {
    /// List registered sources.
    List,
    /// Add a source. Provide exactly one of --path (local) or --git (remote).
    Add {
        /// Unique source name; also the output subdirectory.
        name: String,
        /// Local directory path (for a local source).
        #[arg(long, conflicts_with = "git")]
        path: Option<PathBuf>,
        /// Remote git URL (for a git source).
        #[arg(long)]
        git: Option<String>,
        /// Branch to track for a git source.
        #[arg(long, default_value = "main")]
        branch: String,
    },
    /// Re-pull a git source and rebuild it.
    Resync { name: String },
    /// Remove a source from the registry.
    Remove { name: String },
}

/// Parse args and dispatch. Returns a process-level `Result`.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;

    match cli.command {
        Command::Build => crate::commands::build(&config),
        Command::Clean => crate::commands::clean(&config),
        Command::Init => crate::commands::init(&config),
        Command::Serve { port, watch } => crate::commands::serve(config, port, watch),
        Command::Source { action } => match action {
            SourceAction::List => crate::commands::source_list(),
            SourceAction::Add {
                name,
                path,
                git,
                branch,
            } => crate::commands::source_add(&config, name, path, git, branch),
            SourceAction::Resync { name } => crate::commands::source_resync(&config, name),
            SourceAction::Remove { name } => crate::commands::source_remove(&config, name),
        },
    }
}
