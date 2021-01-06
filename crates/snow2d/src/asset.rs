//! WIP

use std::path::{Path, PathBuf};

// use crate::gfx::tex::Texture2dDrop;

/// Get asset path relative to `assets` directory
pub fn path(path: impl AsRef<Path>) -> PathBuf {
    // TODO: supply appropreate root path
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = PathBuf::from(root).join("assets");
    dir.join(path)
}

// pub fn load_tex(path: impl AsRef<Path>) -> crate::gfx::tex::Result<Texture2dDrop> {
//     Texture2dDrop::from_path(self::path(path))
// }
