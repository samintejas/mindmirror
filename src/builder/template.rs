//! Layout templating via minijinja.
//!
//! Replaces the Go version's duplicated `SCRIPT_HTML` string (copied between
//! `build.go` and `tray.go`). Default layouts are embedded; a user
//! `<config>/templates/*.html` file overrides the embedded one of the same name.
//! The active layout is chosen per page by frontmatter, falling back to the
//! configured default.

use minijinja::{Environment, context};

use crate::config::Config;
use crate::embed::Assets;
use crate::error::{Error, Result};
use crate::paths;

pub struct Templates {
    env: Environment<'static>,
    default_layout: String,
}

impl Templates {
    pub fn load(config: &Config) -> Result<Self> {
        let mut env = Environment::new();

        // Embedded defaults.
        for file in Assets::iter() {
            if let Some(name) = file.strip_prefix("templates/") {
                if name.is_empty() {
                    continue;
                }
                let data = Assets::get(file.as_ref()).expect("embedded asset listed by iter()");
                let src = String::from_utf8_lossy(&data.data).into_owned();
                env.add_template_owned(name.to_string(), src)?;
            }
        }

        // User overrides win.
        let override_dir = paths::config_dir().join("templates");
        if override_dir.is_dir() {
            for entry in
                std::fs::read_dir(&override_dir).map_err(|e| Error::io(&override_dir, e))?
            {
                let path = entry.map_err(|e| Error::io(&override_dir, e))?.path();
                if path.extension().and_then(|e| e.to_str()) == Some("html") {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default()
                        .to_string();
                    let src = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
                    tracing::debug!("override template: {name}");
                    env.add_template_owned(name, src)?;
                }
            }
        }

        Ok(Templates {
            env,
            default_layout: config.defaults.layout.clone(),
        })
    }

    /// Render a page. `layout` (without extension) selects the template; missing
    /// layouts fall back to `default`.
    pub fn render_page(
        &self,
        layout: Option<&str>,
        title: &str,
        body: &str,
        style: &str,
    ) -> Result<String> {
        let layout = layout.unwrap_or(&self.default_layout);
        let name = format!("{layout}.html");
        let template = match self.env.get_template(&name) {
            Ok(t) => t,
            Err(_) => {
                tracing::warn!("layout '{layout}' not found; falling back to 'default'");
                self.env.get_template("default.html")?
            }
        };
        Ok(template.render(context! {
            title => title,
            body => body,
            style => style,
        })?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_with_default_layout() {
        let cfg = Config::default();
        let t = Templates::load(&cfg).unwrap();
        let html = t
            .render_page(None, "My Title", "<p>hi &amp; bye</p>", "water-auto.css")
            .unwrap();
        assert!(html.contains("<title>My Title</title>"));
        assert!(html.contains("<p>hi &amp; bye</p>")); // body passed through |safe
        assert!(html.contains("/styles/water-auto.css"));
    }

    #[test]
    fn selects_named_layout_and_falls_back() {
        let cfg = Config::default();
        let t = Templates::load(&cfg).unwrap();
        let wide = t
            .render_page(Some("wide"), "T", "<p>x</p>", "s.css")
            .unwrap();
        assert!(wide.contains("mm-content-wide"));
        // Unknown layout falls back to default (no panic, valid html).
        let fb = t
            .render_page(Some("nope"), "T", "<p>x</p>", "s.css")
            .unwrap();
        assert!(fb.contains("<title>T</title>"));
        assert!(!fb.contains("mm-content-wide"));
    }
}
