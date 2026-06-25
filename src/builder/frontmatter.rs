//! TOML frontmatter parsing (`+++`-fenced).
//!
//! A page may begin with a `+++ ... +++` TOML block selecting a `layout`, a
//! per-page `style`, and metadata used for the index tree and search index.

use serde::Deserialize;

use crate::error::Result;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrontMatter {
    pub title: Option<String>,
    /// Layout template name (without `.html`); falls back to config default.
    pub layout: Option<String>,
    /// Page stylesheet filename; falls back to config `page_stylesheet`.
    pub style: Option<String>,
    /// ISO date string (kept as text; not interpreted).
    pub date: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Draft pages are skipped during the build.
    #[serde(default)]
    pub draft: bool,
    pub description: Option<String>,
}

/// Split a document into its frontmatter (default if absent) and body.
///
/// Recognizes only a `+++` block at the very start. Returns the remaining
/// markdown body as an owned string (line endings normalized to `\n`).
pub fn parse(content: &str) -> Result<(FrontMatter, String)> {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut lines = content.lines();

    if lines.next().map(str::trim) != Some("+++") {
        return Ok((FrontMatter::default(), content.to_string()));
    }

    let mut fm_lines = Vec::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line.trim() == "+++" {
            closed = true;
            break;
        }
        fm_lines.push(line);
    }

    if !closed {
        // Unterminated fence: treat the whole document as body.
        return Ok((FrontMatter::default(), content.to_string()));
    }

    let fm: FrontMatter = toml::from_str(&fm_lines.join("\n"))?;
    let body = lines.collect::<Vec<_>>().join("\n");
    Ok((fm, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_returns_default() {
        let (fm, body) = parse("# Hello\n\ntext").unwrap();
        assert!(fm.title.is_none());
        assert!(!fm.draft);
        assert_eq!(body, "# Hello\n\ntext");
    }

    #[test]
    fn parses_full_frontmatter() {
        let doc = "+++\n\
            title = \"My Note\"\n\
            layout = \"wide\"\n\
            style = \"tufte.css\"\n\
            tags = [\"a\", \"b\"]\n\
            draft = true\n\
            +++\n\
            # Body\n";
        let (fm, body) = parse(doc).unwrap();
        assert_eq!(fm.title.as_deref(), Some("My Note"));
        assert_eq!(fm.layout.as_deref(), Some("wide"));
        assert_eq!(fm.style.as_deref(), Some("tufte.css"));
        assert_eq!(fm.tags, vec!["a", "b"]);
        assert!(fm.draft);
        assert_eq!(body.trim(), "# Body");
    }

    #[test]
    fn unterminated_fence_is_body() {
        let (fm, body) = parse("+++\ntitle = \"x\"\n# no close").unwrap();
        assert!(fm.title.is_none());
        assert!(body.contains("# no close"));
    }
}
