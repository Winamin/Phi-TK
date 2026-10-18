use anyhow::{bail, Result};
use std::{path::{Path, PathBuf}, sync::OnceLock};

pub static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn ensure_dir(path: PathBuf) -> PathBuf {
    if path.exists() {
        if !path.is_dir() {
            panic!("{} is not a directory", path.display());
        }
    } else {
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}

pub fn output_dir() -> Result<PathBuf> {
    let dir = DATA_DIR.get().unwrap().join("output");
    if dir.exists() {
        if !dir.is_dir() {
            bail!("output directory is not a directory");
        }
    } else {
        std::fs::create_dir(&dir)?;
    }
    Ok(dir)
}

pub fn resolve_output_dir(custom: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = custom.filter(|p| !p.as_os_str().is_empty()) {
        match std::fs::create_dir_all(path) {
            Ok(()) => return Ok(path.to_path_buf()),
            Err(err) => {
                log::warn!(
                    "custom output path unusable ({}: {err}), falling back to the default output dir",
                    path.display()
                );
            }
        }
    }
    output_dir()
}

pub fn respack_dir() -> Result<PathBuf> {
    let dir = CONFIG_DIR.get().unwrap().join("respack");
    if dir.exists() {
        if !dir.is_dir() {
            bail!("resource pack directory is not a directory");
        }
    } else {
        std::fs::create_dir(&dir)?;
    }
    Ok(dir)
}
