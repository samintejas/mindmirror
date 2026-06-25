//! Local-directory source helpers.

use std::path::Path;

use crate::error::Result;
use anyhow::anyhow;

/// Ensure a local source path exists and is a directory.
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("local source path does not exist: {}", path.display()).into());
    }
    if !path.is_dir() {
        return Err(anyhow!("local source path is not a directory: {}", path.display()).into());
    }
    Ok(())
}
