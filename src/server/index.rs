//! Home page: source manager + per-source file tree.
//!
//! Ports the Go `buildTree` algorithm (`server.go:153`), but groups pages by
//! source and uses page titles (from each source's `pages.json`) for link text.

use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};
use minijinja::{Environment, context};

use crate::builder::PageMeta;
use crate::config::Config;
use crate::embed::Assets;
use crate::error::Result;
use crate::source::{ContentSource, SourceRegistry};

use super::AppState;

pub async fn home(State(state): State<AppState>) -> Response {
    match render_home(&state.config) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("home render failed: {e}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("error: {e}"),
            )
                .into_response()
        }
    }
}

fn render_home(config: &Config) -> Result<String> {
    let registry = SourceRegistry::load()?;

    let mut tree = String::new();
    for source in registry.sources() {
        let pages = load_pages(config, &source.name);
        tree.push_str(&render_source_block(source, &pages));
    }
    if registry.is_empty() {
        tree.push_str("<p class=\"empty\">No sources yet — add one above.</p>");
    }

    // Render the embedded (or overridden) index template.
    let template_src = load_index_template(config)?;
    let mut env = Environment::new();
    env.add_template("index", &template_src)?;
    let tmpl = env.get_template("index")?;
    Ok(tmpl.render(context! {
        tree => tree,
        no_sources => registry.is_empty(),
    })?)
}

fn load_index_template(_config: &Config) -> Result<String> {
    // A user override at <config>/templates/index.html wins over the embedded one.
    let override_path = crate::paths::config_dir()
        .join("templates")
        .join("index.html");
    if override_path.is_file() {
        return std::fs::read_to_string(&override_path)
            .map_err(|e| crate::error::Error::io(&override_path, e));
    }
    let data = Assets::get("templates/index.html").expect("embedded index template");
    Ok(String::from_utf8_lossy(&data.data).into_owned())
}

fn load_pages(config: &Config, source_name: &str) -> Vec<PageMeta> {
    let path = super::pages_path(config, source_name);
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn render_source_block(source: &ContentSource, pages: &[PageMeta]) -> String {
    let name = esc(&source.name);
    let mut block = format!(
        "<div class=\"source\" data-source=\"{name}\">\
         <div class=\"source-header\">\
         <span class=\"source-name\">{name}</span>\
         <span class=\"badge badge-{kind}\">{kind}</span>\
         <div class=\"source-actions\">\
         <form class=\"inline\" method=\"post\" action=\"/sources/{name}/resync\">\
         <button title=\"Resync\" aria-label=\"Resync {name}\">⟳</button></form>\
         <form class=\"inline\" method=\"post\" action=\"/sources/{name}/remove\" \
         onsubmit=\"return confirm('Remove source {name}?')\">\
         <button title=\"Remove\" aria-label=\"Remove {name}\">✕</button></form>\
         </div></div>",
        name = name,
        kind = source.kind_label(),
    );
    block.push_str(&build_tree(&source.name, pages));
    block.push_str("</div>");
    block
}

/// Build a nested `<ul>` tree for one source. Mirrors the Go algorithm: track
/// the previous file's folder path and open/close `<ul>`s on divergence.
fn build_tree(source_name: &str, pages: &[PageMeta]) -> String {
    let mut tree = String::from("<ul>");
    let mut current: Vec<&str> = Vec::new();

    for page in pages {
        let parts: Vec<&str> = page.url.split('/').collect();
        if parts.is_empty() {
            continue;
        }

        let mut i = 0;
        while i < current.len() && i < parts.len() && current[i] == parts[i] {
            i += 1;
        }
        for _ in i..current.len() {
            tree.push_str("</ul></li>");
        }
        for part in parts.iter().take(parts.len().saturating_sub(1)).skip(i) {
            tree.push_str(&format!(
                "<li><span class='folder'>{}</span><ul class='nested-item'>",
                esc(part)
            ));
        }

        let href = format!("/{source_name}/{}", page.url);
        tree.push_str(&format!(
            "<li class='html-file'><a href='{}'>{}</a></li>",
            esc_attr(&href),
            esc(&page.title)
        ));

        current = parts[..parts.len() - 1].to_vec();
    }

    for _ in 0..current.len() {
        tree.push_str("</ul></li>");
    }
    tree.push_str("</ul>");
    tree
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;").replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(url: &str, title: &str) -> PageMeta {
        PageMeta {
            url: url.to_string(),
            title: title.to_string(),
            tags: vec![],
            description: None,
            date: None,
        }
    }

    #[test]
    fn builds_nested_tree() {
        let pages = vec![
            page("a/b/deep.html", "Deep"),
            page("a/one.html", "One"),
            page("top.html", "Top"),
        ];
        let html = build_tree("notes", &pages);
        assert!(html.contains("<a href='/notes/a/b/deep.html'>Deep</a>"));
        assert!(html.contains("<span class='folder'>a</span>"));
        assert!(html.contains("<span class='folder'>b</span>"));
        assert!(html.contains("<a href='/notes/top.html'>Top</a>"));
        // Balanced <ul> open/close.
        assert_eq!(html.matches("<ul").count(), html.matches("</ul>").count());
    }

    #[test]
    fn escapes_html_in_titles() {
        let pages = vec![page("x.html", "A & B <script>")];
        let html = build_tree("s", &pages);
        assert!(html.contains("A &amp; B &lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }
}
