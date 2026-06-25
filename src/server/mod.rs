//! HTTP server (axum) + source manager. Replaces the Go `server` package and
//! the dropped system tray: the home page lists and manages content sources.
//!
//! Routes:
//! - `GET  /`                        — home page (source manager + file tree)
//! - `GET  /api/sources`             — JSON list of sources
//! - `GET  /api/pick-folder`         — open a native folder dialog, return its path
//! - `POST /sources/add`             — add a source (HTML form) then redirect
//! - `POST /sources/{name}/resync`   — resync + rebuild a source
//! - `POST /sources/{name}/remove`   — remove a source
//! - `GET  /styles/*` `/scripts/*`   — assets (config override dir or embedded)
//! - fallback                        — built page/image from the output dir

pub mod index;
pub mod pick;
pub mod reload;
pub mod search;
pub mod sources_api;

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::config::Config;
use crate::embed::Assets;
use crate::error::{Error, Result};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    /// Present only in `--watch` mode; rebuilds broadcast a tick here.
    pub reload: Option<broadcast::Sender<()>>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index::home))
        .route("/__reload", get(reload::sse_handler))
        .route("/api/search", get(search::search_json))
        .route("/api/sources", get(sources_api::list_json))
        .route("/api/pick-folder", get(pick::pick_folder))
        .route("/sources/add", post(sources_api::add_form))
        .route("/sources/{name}/resync", post(sources_api::resync_form))
        .route("/sources/{name}/remove", post(sources_api::remove_form))
        .route("/styles/{*path}", get(serve_style))
        .route("/scripts/{*path}", get(serve_script))
        .fallback(serve_content)
        .with_state(state)
}

/// Synchronous entry point used by the CLI: builds a Tokio runtime and serves.
/// With `watch`, performs an initial build and starts the file watcher.
pub fn serve_blocking(config: Config, port: u16, watch: bool) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Other(e.into()))?;
    runtime.block_on(serve(config, port, watch))
}

async fn serve(config: Config, port: u16, watch: bool) -> Result<()> {
    let config = Arc::new(config);

    let reload = if watch {
        // Build everything once up front, then watch local sources.
        if let Err(e) = crate::service::build_all(&config) {
            tracing::warn!("initial build had errors: {e}");
        }
        let (tx, _rx) = broadcast::channel(16);
        crate::watcher::spawn(config.clone(), tx.clone())?;
        Some(tx)
    } else {
        None
    };

    let state = AppState { config, reload };
    let app = router(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("failed to bind {addr}: {e}")))?;
    tracing::info!("serving on http://localhost:{port}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| Error::Other(e.into()))?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}

async fn serve_style(
    State(state): State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    serve_asset(&state.config, "styles", &path)
}

async fn serve_script(
    State(state): State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    serve_asset(&state.config, "scripts", &path)
}

/// Serve a style/script. Resolution order: explicit config override dir →
/// conventional `<config>/{kind}` (where `init` materializes assets) → embedded
/// default.
fn serve_asset(config: &Config, kind: &str, rel: &str) -> Response {
    if has_traversal(rel) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let explicit = match kind {
        "styles" => config.styles.path.as_deref(),
        "scripts" => config.scripts.path.as_deref(),
        _ => None,
    };
    let candidates = [
        explicit.map(Path::to_path_buf),
        Some(crate::paths::config_dir().join(kind)),
    ];
    for dir in candidates.into_iter().flatten() {
        let fp = dir.join(rel);
        if fp.is_file() {
            return file_response(&fp);
        }
    }

    let key = format!("{kind}/{rel}");
    if let Some(file) = Assets::get(&key) {
        let mime = mime_guess::from_path(rel).first_or_octet_stream();
        return (
            [(header::CONTENT_TYPE, mime.as_ref())],
            file.data.into_owned(),
        )
            .into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

/// Fallback: serve a built page or image from the output dir, appending `.html`
/// when the exact path is missing (ported from the Go server).
async fn serve_content(State(state): State<AppState>, uri: Uri) -> Response {
    let decoded = percent_encoding::percent_decode_str(uri.path()).decode_utf8_lossy();
    let rel = decoded.trim_start_matches('/');
    if rel.is_empty() || has_traversal(rel) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let dest = &state.config.output.dest;
    let direct = dest.join(rel);
    let candidate = if direct.is_file() {
        Some(direct)
    } else {
        let html = dest.join(format!("{rel}.html"));
        html.is_file().then_some(html)
    };

    match candidate {
        Some(fp) => file_response(&fp),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn file_response(path: &Path) -> Response {
    match std::fs::read(path) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], bytes).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Reject paths containing `..` or absolute/root components.
fn has_traversal(rel: &str) -> bool {
    Path::new(rel).components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) || rel.contains("..")
}

/// Path under `dest` for a source's `pages.json`, used by the index renderer.
pub(crate) fn pages_path(config: &Config, source_name: &str) -> PathBuf {
    config
        .output
        .dest
        .join(source_name)
        .join(crate::builder::PAGES_FILE)
}
