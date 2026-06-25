//! Search index endpoint. Aggregates every source's `pages.json` into a flat
//! list the client `search.js` filters. Computed on demand so it always
//! reflects the latest build.

use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::builder::PageMeta;
use crate::config::Config;
use crate::source::SourceRegistry;

use super::AppState;

#[derive(Serialize)]
pub struct SearchEntry {
    pub href: String,
    pub title: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub source: String,
}

pub async fn search_json(State(state): State<AppState>) -> Response {
    Json(aggregate(&state.config)).into_response()
}

fn aggregate(config: &Config) -> Vec<SearchEntry> {
    let registry = SourceRegistry::load().unwrap_or_default();
    let mut entries = Vec::new();
    for source in registry.sources() {
        let path = super::pages_path(config, &source.name);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(pages) = serde_json::from_slice::<Vec<PageMeta>>(&bytes) else {
            continue;
        };
        for page in pages {
            entries.push(SearchEntry {
                href: format!("/{}/{}", source.name, page.url),
                title: page.title,
                tags: page.tags,
                description: page.description,
                source: source.name.clone(),
            });
        }
    }
    entries
}
