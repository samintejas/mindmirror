//! Filesystem watcher for `serve --watch`.
//!
//! Watches the content roots of **local** sources (git sources refresh via the
//! manager's resync button, per design). On a debounced batch of *non-access*
//! changes it rebuilds the affected sources incrementally and signals connected
//! browsers to reload via the broadcast channel.
//!
//! Uses `notify-debouncer-full` (which preserves event kinds) and filters out
//! access events: notify's inotify mask includes `OPEN`/`CLOSE_NOWRITE`, so the
//! builder *reading* a file would otherwise re-trigger the watcher in an endless
//! loop. Filtering on `is_access()` breaks that feedback cycle.
//!
//! Replaces the Go `watcher` package, which was never wired in and passed a file
//! path where a directory was expected.

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use tokio::sync::broadcast;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::service;
use crate::source::{ContentSource, SourceRegistry};

/// Spawn the watcher on a dedicated thread. Sources known at startup are
/// watched; sources added later (via the manager) are built on add but require a
/// restart to be watched.
pub fn spawn(config: Arc<Config>, reload_tx: broadcast::Sender<()>) -> Result<()> {
    std::thread::Builder::new()
        .name("mm-watcher".into())
        .spawn(move || {
            if let Err(e) = run(config, reload_tx) {
                tracing::error!("watcher stopped: {e}");
            }
        })
        .map_err(|e| Error::Other(e.into()))?;
    Ok(())
}

fn run(config: Arc<Config>, reload_tx: broadcast::Sender<()>) -> Result<()> {
    let registry = SourceRegistry::load()?;
    let local: Vec<ContentSource> = registry
        .sources()
        .iter()
        .filter(|s| s.is_watchable())
        .cloned()
        .collect();

    if local.is_empty() {
        tracing::info!("watch: no local sources to watch");
        return Ok(());
    }

    let (tx, rx) = channel();
    let mut debouncer =
        new_debouncer(Duration::from_millis(250), None, tx).map_err(|e| Error::Other(e.into()))?;

    for source in &local {
        let root = source.content_root();
        match debouncer.watch(&root, RecursiveMode::Recursive) {
            Ok(()) => tracing::info!("watching {} ({})", root.display(), source.name),
            Err(e) => tracing::warn!("watch failed for {}: {e}", root.display()),
        }
    }

    for batch in rx {
        let events = match batch {
            Ok(events) => events,
            Err(errors) => {
                for e in errors {
                    tracing::error!("watch error: {e}");
                }
                continue;
            }
        };

        // Ignore access-only events (our own reads) to avoid a rebuild loop.
        let mut paths = Vec::new();
        for event in events {
            if event.kind.is_access() {
                continue;
            }
            paths.extend(event.event.paths.iter().cloned());
        }
        if paths.is_empty() {
            continue;
        }

        let mut rebuilt = HashSet::new();
        let mut changed = false;
        for source in &local {
            let root = source.content_root();
            if paths.iter().any(|p| p.starts_with(&root)) && rebuilt.insert(source.name.clone()) {
                match service::build_one(&config, source) {
                    Ok(r) => {
                        if r.generated > 0 || r.images > 0 {
                            changed = true;
                        }
                        tracing::info!(
                            "rebuilt {}: {} generated, {} skipped",
                            r.name,
                            r.generated,
                            r.skipped
                        );
                    }
                    Err(e) => tracing::error!("rebuild '{}' failed: {e}", source.name),
                }
            }
        }

        // Only reload the browser when output actually changed.
        if changed {
            let _ = reload_tx.send(());
        }
    }

    Ok(())
}
