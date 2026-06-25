//! The build pipeline: markdown → themed HTML, per source.
//!
//! Replaces the Go `builder` package. Each source builds into its own
//! `dest/<name>/` subtree, with a content-hash manifest for incremental builds
//! and a `pages.json` of page metadata consumed by the server (index tree) and
//! search index.

pub mod assets;
pub mod frontmatter;
pub mod manifest;
pub mod markdown;
pub mod template;

use std::path::Path;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::source::ContentSource;
use manifest::Manifest;
use template::Templates;

pub const PAGES_FILE: &str = "pages.json";

/// Per-page metadata persisted to `pages.json`, used for the index and search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    /// Source-relative URL, e.g. `Data Science/Tools.html`.
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

/// Outcome of building one source.
#[derive(Debug, Default)]
pub struct SourceReport {
    pub name: String,
    pub generated: usize,
    pub skipped: usize,
    pub images: usize,
    pub drafts: usize,
    pub pages: Vec<PageMeta>,
}

pub struct Builder<'a> {
    config: &'a Config,
    templates: Templates,
}

impl<'a> Builder<'a> {
    pub fn new(config: &'a Config) -> Result<Self> {
        Ok(Builder {
            config,
            templates: Templates::load(config)?,
        })
    }

    pub fn build_all(&self, sources: &[ContentSource]) -> Result<Vec<SourceReport>> {
        sources.iter().map(|s| self.build_source(s)).collect()
    }

    pub fn build_source(&self, source: &ContentSource) -> Result<SourceReport> {
        let root = source.content_root();
        if !root.is_dir() {
            return Err(Error::Other(anyhow!(
                "content for source '{}' is missing at {} — add or resync it first",
                source.name,
                root.display()
            )));
        }

        let out_dir = self.config.output.dest.join(&source.name);
        std::fs::create_dir_all(&out_dir).map_err(|e| Error::io(&out_dir, e))?;

        let mut manifest = Manifest::load(&out_dir)?;
        let mut report = SourceReport {
            name: source.name.clone(),
            ..Default::default()
        };

        for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let rel = match path.strip_prefix(&root) {
                Ok(r) => r.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };

            let is_md = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("md"));

            if is_md {
                self.process_markdown(path, &rel, &out_dir, &mut manifest, &mut report)?;
            } else if assets::is_image(path) {
                self.process_image(path, &rel, &out_dir, &mut manifest, &mut report)?;
            }
        }

        report.pages.sort_by(|a, b| a.url.cmp(&b.url));
        let pages_path = out_dir.join(PAGES_FILE);
        let json = serde_json::to_vec_pretty(&report.pages)?;
        std::fs::write(&pages_path, json).map_err(|e| Error::io(&pages_path, e))?;

        manifest.save(&out_dir)?;
        Ok(report)
    }

    fn process_markdown(
        &self,
        path: &Path,
        rel: &str,
        out_dir: &Path,
        manifest: &mut Manifest,
        report: &mut SourceReport,
    ) -> Result<()> {
        let bytes = std::fs::read(path).map_err(|e| Error::io(path, e))?;
        let hash = manifest::hash(&bytes);
        let content = String::from_utf8_lossy(&bytes);
        let (fm, body_md) = frontmatter::parse(&content)?;

        let out_path = out_dir.join(html_rel(rel));

        if fm.draft {
            report.drafts += 1;
            // Remove stale output if a page was previously published.
            let _ = std::fs::remove_file(&out_path);
            return Ok(());
        }

        let title = fm.title.clone().unwrap_or_else(|| title_from_rel(rel));
        let meta = PageMeta {
            url: html_rel(rel),
            title: title.clone(),
            tags: fm.tags.clone(),
            description: fm.description.clone(),
            date: fm.date.clone(),
        };

        let key = format!("md:{rel}");
        if manifest.is_unchanged(&key, &hash) && out_path.exists() {
            report.skipped += 1;
            report.pages.push(meta);
            return Ok(());
        }

        let body_html = markdown::render(&body_md);
        let style = fm
            .style
            .clone()
            .unwrap_or_else(|| self.config.styles.page_stylesheet.clone());
        let html = self
            .templates
            .render_page(fm.layout.as_deref(), &title, &body_html, &style)?;

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        std::fs::write(&out_path, html).map_err(|e| Error::io(&out_path, e))?;

        manifest.update(key, hash);
        report.generated += 1;
        report.pages.push(meta);
        Ok(())
    }

    fn process_image(
        &self,
        path: &Path,
        rel: &str,
        out_dir: &Path,
        manifest: &mut Manifest,
        report: &mut SourceReport,
    ) -> Result<()> {
        let bytes = std::fs::read(path).map_err(|e| Error::io(path, e))?;
        let hash = manifest::hash(&bytes);
        let out_path = out_dir.join(rel);
        let key = format!("img:{rel}");
        if manifest.is_unchanged(&key, &hash) && out_path.exists() {
            report.skipped += 1;
            return Ok(());
        }
        assets::copy_file(path, &out_path)?;
        manifest.update(key, hash);
        report.images += 1;
        Ok(())
    }
}

/// `dir/note.md` → `dir/note.html`; other paths unchanged.
fn html_rel(rel: &str) -> String {
    match rel.strip_suffix(".md") {
        Some(stem) => format!("{stem}.html"),
        None => rel.to_string(),
    }
}

fn title_from_rel(rel: &str) -> String {
    Path::new(rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(rel)
        .to_string()
}
