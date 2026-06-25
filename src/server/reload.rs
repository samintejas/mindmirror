//! Server-Sent Events endpoint for live reload.
//!
//! `reload.js` (injected into every page) connects to `/__reload`. In watch
//! mode each rebuild broadcasts a tick that becomes a `reload` SSE event; the
//! client then reloads. Outside watch mode the endpoint 404s and the client
//! quietly gives up.

use std::convert::Infallible;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use super::AppState;

pub async fn sse_handler(State(state): State<AppState>) -> Response {
    let Some(tx) = state.reload.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let stream = BroadcastStream::new(tx.subscribe())
        .map(|_result| Ok::<Event, Infallible>(Event::default().event("reload").data("changed")));

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}
