//! Source management endpoints. Mutating actions use HTML form POSTs (so the
//! manager works without JavaScript) and redirect back to `/`; a read-only JSON
//! listing is provided for tooling. All operations delegate to [`crate::service`].

use axum::Json;
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};

use crate::service;
use crate::source::{ContentSource, SourceRegistry};

use super::AppState;

#[derive(Serialize)]
pub struct SourceView {
    pub name: String,
    pub kind: String,
    pub root: String,
    pub watchable: bool,
}

pub async fn list_json() -> Response {
    match SourceRegistry::load() {
        Ok(reg) => {
            let views: Vec<SourceView> = reg
                .sources()
                .iter()
                .map(|s| SourceView {
                    name: s.name.clone(),
                    kind: s.kind_label().to_string(),
                    root: s.content_root().display().to_string(),
                    watchable: s.is_watchable(),
                })
                .collect();
            Json(views).into_response()
        }
        Err(e) => error_response(&e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct AddForm {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub branch: String,
}

pub async fn add_form(State(state): State<AppState>, Form(form): Form<AddForm>) -> Response {
    let source = match form.kind.as_str() {
        "local" => {
            if form.path.trim().is_empty() {
                return error_response("local source requires a path");
            }
            ContentSource::local(form.name.trim(), form.path.trim())
        }
        "git" => {
            if form.url.trim().is_empty() {
                return error_response("git source requires a url");
            }
            let branch = if form.branch.trim().is_empty() {
                "main"
            } else {
                form.branch.trim()
            };
            ContentSource::git(form.name.trim(), form.url.trim(), branch)
        }
        other => return error_response(&format!("unknown source kind '{other}'")),
    };

    let source = match source {
        Ok(s) => s,
        Err(e) => return error_response(&e.to_string()),
    };

    match service::add_source(&state.config, source) {
        Ok(_) => Redirect::to("/").into_response(),
        Err(e) => error_response(&e.to_string()),
    }
}

pub async fn resync_form(State(state): State<AppState>, Path(name): Path<String>) -> Response {
    match service::resync_source(&state.config, &name) {
        Ok(_) => Redirect::to("/").into_response(),
        Err(e) => error_response(&e.to_string()),
    }
}

pub async fn remove_form(State(state): State<AppState>, Path(name): Path<String>) -> Response {
    match service::remove_source(&state.config, &name) {
        Ok(_) => Redirect::to("/").into_response(),
        Err(e) => error_response(&e.to_string()),
    }
}

fn error_response(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, format!("error: {msg}")).into_response()
}
