//! Typed, defaulted TOML configuration.
//!
//! Replaces the Go version's untyped Viper access (`viper.GetString("app...")`
//! sprinkled across ~20 call sites). Every field has a sensible default, so the
//! tool runs even with no config file present. Discovery order mirrors the Go
//! version (`cmd/root.go`): explicit `--config` → `$XDG_CONFIG_HOME/mindmirror`
//! → `/etc/mindmirror` → current directory.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::paths;

pub const CONFIG_FILENAME: &str = "mindmirror.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// HTTP port for `serve`.
    pub port: u16,
    pub output: Output,
    pub styles: Styles,
    pub scripts: Scripts,
    pub defaults: Defaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Output {
    /// Root directory for generated HTML. Each source builds into `dest/<name>`.
    pub dest: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Styles {
    /// Optional override directory for stylesheets. When unset, embedded
    /// default themes are used.
    pub path: Option<PathBuf>,
    /// Default page theme stylesheet. Empty (the default) renders pages with
    /// the built-in Tailwind `prose` typography; a file name selects a legacy
    /// theme. Overridable per page via frontmatter `style`.
    pub page_stylesheet: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Scripts {
    /// Optional override directory for client scripts. Embedded defaults used
    /// when unset.
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Defaults {
    /// Default layout template name; overridable per page via frontmatter `layout`.
    pub layout: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 8080,
            output: Output::default(),
            styles: Styles::default(),
            scripts: Scripts::default(),
            defaults: Defaults::default(),
        }
    }
}

impl Default for Output {
    fn default() -> Self {
        Output {
            dest: paths::cache_dir().join("public"),
        }
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Defaults {
            layout: "default".to_string(),
        }
    }
}

impl Config {
    /// Load configuration. With `explicit`, that file must exist and parse.
    /// Otherwise discover one; if none is found, fall back to defaults.
    pub fn load(explicit: Option<&Path>) -> Result<Config> {
        let chosen = match explicit {
            Some(p) => Some(p.to_path_buf()),
            None => Self::discover(),
        };

        let mut config = match chosen {
            Some(path) => {
                let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
                tracing::debug!("loaded config from {}", path.display());
                toml::from_str(&text)?
            }
            None => {
                tracing::debug!("no config file found; using defaults");
                Config::default()
            }
        };

        config.expand_paths();
        Ok(config)
    }

    fn discover() -> Option<PathBuf> {
        let candidates = [
            paths::config_dir().join(CONFIG_FILENAME),
            PathBuf::from("/etc/mindmirror").join(CONFIG_FILENAME),
            PathBuf::from(CONFIG_FILENAME),
        ];
        candidates.into_iter().find(|p| p.is_file())
    }

    fn expand_paths(&mut self) {
        self.output.dest = paths::expand_tilde(&self.output.dest);
        if let Some(p) = &self.styles.path {
            self.styles.path = Some(paths::expand_tilde(p));
        }
        if let Some(p) = &self.scripts.path {
            self.scripts.path = Some(paths::expand_tilde(p));
        }
    }

    /// Serialize to a documented TOML string for `mindmirror init`.
    pub fn to_toml_string(&self) -> Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sensible() {
        let c = Config::default();
        assert_eq!(c.port, 8080);
        assert_eq!(c.styles.page_stylesheet, "");
        assert_eq!(c.defaults.layout, "default");
        assert!(c.styles.path.is_none());
    }

    #[test]
    fn partial_config_fills_defaults() {
        // Only `port` is specified; everything else must default.
        let c: Config = toml::from_str("port = 9090\n").unwrap();
        assert_eq!(c.port, 9090);
        assert_eq!(c.styles.page_stylesheet, "");
        assert_eq!(c.defaults.layout, "default");
    }

    #[test]
    fn full_config_roundtrips() {
        let toml_src = r#"
            port = 3000
            [output]
            dest = "/tmp/out"
            [styles]
            page_stylesheet = "tufte.css"
            [defaults]
            layout = "wide"
        "#;
        let c: Config = toml::from_str(toml_src).unwrap();
        assert_eq!(c.port, 3000);
        assert_eq!(c.output.dest, PathBuf::from("/tmp/out"));
        assert_eq!(c.styles.page_stylesheet, "tufte.css");
        assert_eq!(c.defaults.layout, "wide");
        // Re-serialize and re-parse to confirm stability.
        let back = c.to_toml_string().unwrap();
        let c2: Config = toml::from_str(&back).unwrap();
        assert_eq!(c2.port, 3000);
    }
}
