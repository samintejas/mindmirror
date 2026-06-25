//! Native folder picker. The server runs on the user's own machine, so it can
//! open the desktop's directory chooser and hand the chosen *absolute* path back
//! to the page — something a browser file input cannot do (it hides real paths).
//!
//! `GET /api/pick-folder` →
//! - `200 {"path": "/abs/path"}` when a folder was chosen,
//! - `204 No Content` when the dialog was cancelled,
//! - `501 Not Implemented` when no picker is installed (type the path instead).

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tokio::process::Command;

/// Picker commands tried in order. Each prints the chosen directory to stdout
/// and exits non-zero when the user cancels.
const PICKERS: &[(&str, &[&str])] = &[
    (
        "zenity",
        &[
            "--file-selection",
            "--directory",
            "--title",
            "Select notes folder",
        ],
    ),
    ("kdialog", &["--getexistingdirectory", "."]),
];

#[derive(Serialize)]
struct Picked {
    path: String,
}

pub async fn pick_folder() -> Response {
    for (cmd, args) in PICKERS {
        match Command::new(cmd).args(*args).output().await {
            Ok(out) if out.status.success() => {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return if path.is_empty() {
                    StatusCode::NO_CONTENT.into_response()
                } else {
                    Json(Picked { path }).into_response()
                };
            }
            // Ran but exited non-zero: the user cancelled the dialog.
            Ok(_) => return StatusCode::NO_CONTENT.into_response(),
            // Not installed: try the next picker.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                tracing::warn!("folder picker '{cmd}' failed: {e}");
                continue;
            }
        }
    }

    (
        StatusCode::NOT_IMPLEMENTED,
        "No folder picker found. Install zenity or kdialog, or type the path manually.",
    )
        .into_response()
}
