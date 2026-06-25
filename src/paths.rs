//! Centralized XDG path discovery.
//!
//! Replaces the Go version's ad-hoc `os.UserConfigDir()` usage with a single
//! place that resolves config/data/cache locations and expands `~`.

use directories::ProjectDirs;
use std::path::{Path, PathBuf};

const QUALIFIER: &str = "dev";
const ORG: &str = "samin";
const APP: &str = "mindmirror";

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORG, APP)
}

/// `$XDG_CONFIG_HOME/mindmirror` (e.g. `~/.config/mindmirror`).
pub fn config_dir() -> PathBuf {
    project_dirs()
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| fallback(".config"))
}

/// `$XDG_DATA_HOME/mindmirror` — home for the sources registry and git caches.
pub fn data_dir() -> PathBuf {
    project_dirs()
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| fallback(".local/share"))
}

/// `$XDG_CACHE_HOME/mindmirror` — default build output root.
pub fn cache_dir() -> PathBuf {
    project_dirs()
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(|| fallback(".cache"))
}

/// Where cloned git sources live: `<data>/sources/<name>`.
pub fn git_cache_dir(name: &str) -> PathBuf {
    data_dir().join("sources").join(name)
}

/// The server-managed sources registry file.
pub fn sources_file() -> PathBuf {
    data_dir().join("sources.toml")
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn fallback(suffix: &str) -> PathBuf {
    home().join(suffix).join(APP)
}

/// Expand a leading `~` to the user's home directory.
pub fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.as_os_str().to_string_lossy();
    if let Some(rest) = s.strip_prefix("~/") {
        home().join(rest)
    } else if s == "~" {
        home()
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_leading_tilde() {
        // SAFETY: single-threaded test; we set HOME for the duration.
        unsafe { std::env::set_var("HOME", "/home/tester") };
        assert_eq!(
            expand_tilde(Path::new("~/notes")),
            PathBuf::from("/home/tester/notes")
        );
        assert_eq!(expand_tilde(Path::new("~")), PathBuf::from("/home/tester"));
        // Non-tilde paths are untouched.
        assert_eq!(
            expand_tilde(Path::new("/abs/path")),
            PathBuf::from("/abs/path")
        );
    }
}
