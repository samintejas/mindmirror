//! Source + build operations shared by the CLI (`commands`) and the web API
//! (`server::sources_api`), so neither duplicates the registry/build logic.

use crate::builder::{Builder, SourceReport};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::source::{ContentSource, SourceRegistry};

/// Materialize, register, and build a new source.
pub fn add_source(config: &Config, source: ContentSource) -> Result<SourceReport> {
    source.sync()?;
    let mut registry = SourceRegistry::load()?;
    registry.add(source.clone())?;
    registry.save()?;
    build_one(config, &source)
}

/// Re-pull (git) or re-validate (local) a source and rebuild it.
pub fn resync_source(config: &Config, name: &str) -> Result<SourceReport> {
    let registry = SourceRegistry::load()?;
    let source = registry
        .get(name)
        .ok_or_else(|| Error::SourceNotFound(name.to_string()))?
        .clone();
    source.sync()?;
    build_one(config, &source)
}

/// Remove a source from the registry and drop its output + git cache.
pub fn remove_source(config: &Config, name: &str) -> Result<ContentSource> {
    let mut registry = SourceRegistry::load()?;
    let removed = registry.remove(name)?;
    registry.save()?;
    let _ = std::fs::remove_dir_all(config.output.dest.join(&removed.name));
    let _ = std::fs::remove_dir_all(crate::paths::git_cache_dir(&removed.name));
    Ok(removed)
}

pub fn build_one(config: &Config, source: &ContentSource) -> Result<SourceReport> {
    Builder::new(config)?.build_source(source)
}

pub fn build_all(config: &Config) -> Result<Vec<SourceReport>> {
    let registry = SourceRegistry::load()?;
    Builder::new(config)?.build_all(registry.sources())
}
