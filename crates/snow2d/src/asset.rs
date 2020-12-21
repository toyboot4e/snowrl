//! WIP

use std::path::{Path, PathBuf};

use crate::gfx::tex::Texture2dDrop;

pub fn path(path: impl AsRef<Path>) -> PathBuf {
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = PathBuf::from(root).join("assets");
    dir.join(path)
}

pub fn load_tex(path: impl AsRef<Path>) -> crate::gfx::tex::Result<Texture2dDrop> {
    let root = std::env::var("CARGO_MANIFEST").unwrap();
    let dir = PathBuf::from(root).join("assets");
    Texture2dDrop::from_path(dir.join(path))
}
