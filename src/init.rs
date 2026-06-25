//! `mindmirror init`: scaffold a config file and materialize default assets into
//! the user config directory so they can be customized.

use std::path::Path;

use crate::config::{CONFIG_FILENAME, Config};
use crate::embed::Assets;
use crate::error::{Error, Result};
use crate::paths;

pub fn run(config: &Config) -> Result<()> {
    let config_dir = paths::config_dir();
    std::fs::create_dir_all(&config_dir).map_err(|e| Error::io(&config_dir, e))?;

    // Write a config file if one does not already exist.
    let config_path = config_dir.join(CONFIG_FILENAME);
    if config_path.exists() {
        println!("Config already exists at {}", config_path.display());
    } else {
        std::fs::write(&config_path, config.to_toml_string()?)
            .map_err(|e| Error::io(&config_path, e))?;
        println!("Wrote {}", config_path.display());
    }

    // Materialize embedded assets (styles/scripts/templates) for customization.
    let mut count = 0;
    for file in Assets::iter() {
        let dest = config_dir.join(file.as_ref());
        if dest.exists() {
            continue;
        }
        let data = Assets::get(file.as_ref()).expect("embedded asset listed by iter()");
        write_file(&dest, &data.data)?;
        count += 1;
    }
    println!(
        "Materialized {count} default asset file(s) into {}",
        config_dir.display()
    );
    println!(
        "Edit {} then add a source with `mindmirror source add`.",
        config_path.display()
    );
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    std::fs::write(path, bytes).map_err(|e| Error::io(path, e))
}
