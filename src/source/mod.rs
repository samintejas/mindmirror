//! Content sources: local directories or remote git repositories.
//!
//! This realizes the "redesign" the Go config hinted at (its unused
//! `sourcetype: local|git`). Sources are managed from the web home page and the
//! `mindmirror source` CLI, persisted to a registry at `<data>/sources.toml`.

pub mod git;
pub mod local;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::paths;

/// A single content source. `name` is unique and also the output subdirectory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentSource {
    pub name: String,
    #[serde(flatten)]
    pub kind: SourceKind,
}

/// Discriminated by a `type` key in TOML (`local` or `git`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SourceKind {
    Local { path: PathBuf },
    Git { url: String, branch: String },
}

impl ContentSource {
    pub fn local(name: impl Into<String>, path: impl Into<PathBuf>) -> Result<Self> {
        let name = name.into();
        validate_name(&name)?;
        Ok(ContentSource {
            name,
            kind: SourceKind::Local { path: path.into() },
        })
    }

    pub fn git(
        name: impl Into<String>,
        url: impl Into<String>,
        branch: impl Into<String>,
    ) -> Result<Self> {
        let name = name.into();
        validate_name(&name)?;
        Ok(ContentSource {
            name,
            kind: SourceKind::Git {
                url: url.into(),
                branch: branch.into(),
            },
        })
    }

    /// The directory holding this source's markdown/images. For git sources
    /// this is the local clone cache; for local sources it is the path itself.
    pub fn content_root(&self) -> PathBuf {
        match &self.kind {
            SourceKind::Local { path } => path.clone(),
            SourceKind::Git { .. } => paths::git_cache_dir(&self.name),
        }
    }

    /// Only local sources are watched by the file watcher; git sources refresh
    /// via `resync`.
    pub fn is_watchable(&self) -> bool {
        matches!(self.kind, SourceKind::Local { .. })
    }

    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            SourceKind::Local { .. } => "local",
            SourceKind::Git { .. } => "git",
        }
    }

    /// Make the content available locally: validate a local dir, or clone/pull a
    /// git source into its cache.
    pub fn sync(&self) -> Result<()> {
        match &self.kind {
            SourceKind::Local { path } => local::ensure_dir(path),
            SourceKind::Git { url, branch } => git::sync(&self.name, url, branch),
        }
    }
}

/// Reject names that could escape the output directory or break the URL space.
pub fn validate_name(name: &str) -> Result<()> {
    let bad = name.is_empty()
        || name == "."
        || name == ".."
        || name.contains("..")
        || name.contains('/')
        || name.contains('\\')
        || name.contains(std::path::MAIN_SEPARATOR);
    if bad {
        return Err(Error::InvalidSourceName(name.to_string()));
    }
    Ok(())
}

/// The persisted list of sources (`<data>/sources.toml`).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SourceRegistry {
    #[serde(default, rename = "source")]
    sources: Vec<ContentSource>,
}

impl SourceRegistry {
    /// Load the registry, returning an empty one if the file does not exist.
    pub fn load() -> Result<Self> {
        let path = paths::sources_file();
        match std::fs::read_to_string(&path) {
            Ok(text) => Ok(toml::from_str(&text)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(Error::io(&path, e)),
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::sources_file();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        let text = toml::to_string_pretty(self)?;
        std::fs::write(&path, text).map_err(|e| Error::io(&path, e))
    }

    pub fn sources(&self) -> &[ContentSource] {
        &self.sources
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&ContentSource> {
        self.sources.iter().find(|s| s.name == name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Add a source. Errors if the name already exists.
    pub fn add(&mut self, source: ContentSource) -> Result<()> {
        if self.contains(&source.name) {
            return Err(Error::SourceExists(source.name));
        }
        self.sources.push(source);
        Ok(())
    }

    /// Remove and return a source by name. Errors if not present.
    pub fn remove(&mut self, name: &str) -> Result<ContentSource> {
        match self.sources.iter().position(|s| s.name == name) {
            Some(i) => Ok(self.sources.remove(i)),
            None => Err(Error::SourceNotFound(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_toml_roundtrips() {
        let mut reg = SourceRegistry::default();
        reg.add(ContentSource::local("notes", "/home/x/notes").unwrap())
            .unwrap();
        reg.add(ContentSource::git("blog", "https://example.com/b.git", "main").unwrap())
            .unwrap();

        let text = toml::to_string_pretty(&reg).unwrap();
        let back: SourceRegistry = toml::from_str(&text).unwrap();

        assert_eq!(back.sources().len(), 2);
        assert_eq!(
            back.get("notes").unwrap().kind,
            SourceKind::Local {
                path: "/home/x/notes".into()
            }
        );
        assert_eq!(
            back.get("blog").unwrap().kind,
            SourceKind::Git {
                url: "https://example.com/b.git".into(),
                branch: "main".into()
            }
        );
    }

    #[test]
    fn rejects_duplicate_and_missing() {
        let mut reg = SourceRegistry::default();
        reg.add(ContentSource::local("a", "/a").unwrap()).unwrap();
        assert!(matches!(
            reg.add(ContentSource::local("a", "/b").unwrap()),
            Err(Error::SourceExists(_))
        ));
        assert!(matches!(reg.remove("nope"), Err(Error::SourceNotFound(_))));
    }

    #[test]
    fn rejects_bad_names() {
        assert!(ContentSource::local("", "/a").is_err());
        assert!(ContentSource::local("a/b", "/a").is_err());
        assert!(ContentSource::local("..", "/a").is_err());
        assert!(ContentSource::local("a..b", "/a").is_err());
        assert!(ContentSource::local("ok-name_1", "/a").is_ok());
    }

    #[test]
    fn content_root_for_git_is_cache() {
        let g = ContentSource::git("blog", "u", "main").unwrap();
        assert_eq!(g.content_root(), paths::git_cache_dir("blog"));
        assert!(!g.is_watchable());

        let l = ContentSource::local("n", "/n").unwrap();
        assert_eq!(l.content_root(), PathBuf::from("/n"));
        assert!(l.is_watchable());
    }
}
