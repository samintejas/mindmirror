//! Static file helpers for the build: image detection and copying.

use std::path::Path;

use crate::error::{Error, Result};

const IMAGE_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "avif",
];

/// True if `path` has a recognized image extension (case-insensitive).
/// Broader than the Go version, which only handled png/jpg/jpeg.
pub fn is_image(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => IMAGE_EXTS.contains(&ext.to_ascii_lowercase().as_str()),
        None => false,
    }
}

/// Copy a file, creating parent directories as needed.
pub fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    std::fs::copy(src, dst).map_err(|e| Error::io(src, e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_images_case_insensitively() {
        assert!(is_image(Path::new("a/b.PNG")));
        assert!(is_image(Path::new("x.jpeg")));
        assert!(is_image(Path::new("x.webp")));
        assert!(!is_image(Path::new("x.md")));
        assert!(!is_image(Path::new("noext")));
    }
}
