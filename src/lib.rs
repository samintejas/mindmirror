//! mindmirror — a minimal static-site generator for markdown notes.
//!
//! Library crate; the binary (`src/main.rs`) is a thin wrapper over [`run`].

pub mod builder;
pub mod cli;
pub mod commands;
pub mod config;
pub mod embed;
pub mod error;
pub mod init;
pub mod paths;
pub mod server;
pub mod service;
pub mod source;
pub mod watcher;

pub use cli::run;
pub use error::{Error, Result};

/// Initialize tracing from `RUST_LOG` (default: `info`).
pub fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
}
