//! Content-hash build manifest (replaces the Go version's `content.modtime`).
//!
//! Maps each source-relative path to a blake3 hash of its bytes, so unchanged
//! files are skipped on rebuild. Hash-based (not mtime-based) so it survives git
//! checkouts and rsync that rewrite timestamps.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub const MANIFEST_FILE: &str = ".manifest.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    hashes: BTreeMap<String, String>,
}

impl Manifest {
    /// Load a manifest from `dir/.manifest.json`, or an empty one if absent.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(MANIFEST_FILE);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(Error::io(&path, e)),
        }
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let path = dir.join(MANIFEST_FILE);
        let bytes = serde_json::to_vec_pretty(self)?;
        std::fs::write(&path, bytes).map_err(|e| Error::io(&path, e))
    }

    /// True if `key` is already recorded with the same hash.
    pub fn is_unchanged(&self, key: &str, hash: &str) -> bool {
        self.hashes.get(key).map(String::as_str) == Some(hash)
    }

    pub fn update(&mut self, key: impl Into<String>, hash: impl Into<String>) {
        self.hashes.insert(key.into(), hash.into());
    }
}

/// Hex blake3 hash of a byte slice.
pub fn hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_changes() {
        let mut m = Manifest::default();
        let h = hash(b"hello");
        assert!(!m.is_unchanged("a.md", &h));
        m.update("a.md", &h);
        assert!(m.is_unchanged("a.md", &h));
        assert!(!m.is_unchanged("a.md", &hash(b"world")));
    }
}
