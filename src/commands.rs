//! Command handlers. Thin glue between the CLI and the library modules; the
//! source/build logic lives in [`crate::service`] so the web API shares it.

use std::path::PathBuf;

use anyhow::anyhow;

use crate::builder::SourceReport;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::server;
use crate::service;
use crate::source::{ContentSource, SourceRegistry};

pub fn build(config: &Config) -> Result<()> {
    let registry = SourceRegistry::load()?;
    if registry.is_empty() {
        println!("No sources registered. Add one with `mindmirror source add`.");
        return Ok(());
    }
    for report in service::build_all(config)? {
        print_report(&report);
    }
    Ok(())
}

pub fn clean(config: &Config) -> Result<()> {
    let dest = &config.output.dest;
    if dest.exists() {
        std::fs::remove_dir_all(dest).map_err(|e| Error::io(dest, e))?;
        println!("Removed {}", dest.display());
    } else {
        println!("Nothing to clean ({} does not exist)", dest.display());
    }
    Ok(())
}

pub fn init(config: &Config) -> Result<()> {
    crate::init::run(config)
}

pub fn serve(config: Config, port: Option<u16>, watch: bool) -> Result<()> {
    let port = port.unwrap_or(config.port);
    server::serve_blocking(config, port, watch)
}

pub fn source_list() -> Result<()> {
    let registry = SourceRegistry::load()?;
    if registry.is_empty() {
        println!("No sources registered. Add one with `mindmirror source add`.");
        return Ok(());
    }
    for s in registry.sources() {
        println!(
            "{:<20} {:<6} {}",
            s.name,
            s.kind_label(),
            s.content_root().display()
        );
    }
    Ok(())
}

pub fn source_add(
    config: &Config,
    name: String,
    path: Option<PathBuf>,
    git: Option<String>,
    branch: String,
) -> Result<()> {
    let source = match (path, git) {
        (Some(path), None) => ContentSource::local(name, path)?,
        (None, Some(url)) => ContentSource::git(name, url, branch)?,
        (None, None) => {
            return Err(Error::Other(anyhow!(
                "provide either --path (local) or --git (remote)"
            )));
        }
        (Some(_), Some(_)) => {
            return Err(Error::Other(anyhow!(
                "--path and --git are mutually exclusive"
            )));
        }
    };
    let label = source.kind_label();
    let display_name = source.name.clone();
    let report = service::add_source(config, source)?;
    println!("Added {label} source '{display_name}'.");
    print_report(&report);
    Ok(())
}

pub fn source_resync(config: &Config, name: String) -> Result<()> {
    let report = service::resync_source(config, &name)?;
    println!("Resynced source '{name}'.");
    print_report(&report);
    Ok(())
}

pub fn source_remove(config: &Config, name: String) -> Result<()> {
    let removed = service::remove_source(config, &name)?;
    println!("Removed source '{}'.", removed.name);
    Ok(())
}

fn print_report(r: &SourceReport) {
    println!(
        "{}: {} generated, {} skipped, {} images, {} drafts",
        r.name, r.generated, r.skipped, r.images, r.drafts
    );
}
