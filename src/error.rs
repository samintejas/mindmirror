//! Typed error model for mindmirror.
//!
//! Replaces the Go version's swallowed errors (`config/config.go` had empty
//! `if err != nil {}` blocks) and `log.Fatal` calls with a single error enum
//! that carries context. `anyhow::Error` is accepted via the `Other` variant so
//! call sites can use `.context(...)?` freely and still funnel into this type.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("source '{0}' not found")]
    SourceNotFound(String),

    #[error("source '{0}' already exists")]
    SourceExists(String),

    #[error("invalid source name '{0}': must be non-empty and contain no path separators or '..'")]
    InvalidSourceName(String),

    #[error("i/o error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    PlainIo(#[from] std::io::Error),

    #[error("failed to parse TOML: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("failed to serialize TOML: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("template error: {0}")]
    Template(#[from] minijinja::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// Wrap an [`std::io::Error`] with the path it happened on.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
