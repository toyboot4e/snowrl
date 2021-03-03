/*!
Font resource handling
*/

use std::path::PathBuf;

pub use rokol::fons::{fontstash::FontIx, FontTexture};

// --------------------------------------------------------------------------------
// Desc types for loading

/// Bytes loading description
#[derive(Debug)]
pub enum LoadDesc<'a> {
    Path(PathBuf),
    Mem(&'a [u8]),
}

impl<'a> From<PathBuf> for LoadDesc<'a> {
    fn from(x: PathBuf) -> Self {
        Self::Path(x)
    }
}

impl<'a> From<&'a [u8]> for LoadDesc<'a> {
    fn from(x: &'a [u8]) -> Self {
        Self::Mem(x)
    }
}

#[derive(Debug)]
pub struct FontSetDesc<'a> {
    pub name: String,
    pub regular: FontDesc<'a>,
    pub bold: Option<FontDesc<'a>>,
    pub italic: Option<FontDesc<'a>>,
}

#[derive(Debug)]
pub struct FontDesc<'a> {
    pub name: String,
    pub load: LoadDesc<'a>,
}

// --------------------------------------------------------------------------------
// Font handles

#[derive(Debug, Clone, Copy)]
pub struct FontHandle {
    pub ix: FontIx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFace {
    Regular,
    Bold,
    Italic,
}

#[derive(Debug, Clone)]
pub struct FontSetHandle {
    pub name: String,
    pub regular: FontHandle,
    pub bold: Option<FontHandle>,
    pub italic: Option<FontHandle>,
}
