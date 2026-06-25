//! Embedded default assets (templates, styles, scripts).
//!
//! Bundled into the binary via `rust-embed` so mindmirror works out-of-the-box.
//! User config directories may override any of these at runtime (see
//! `builder::template` and the server's static handlers).

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;
