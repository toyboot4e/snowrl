use std::path::PathBuf;

pub use rokol::fons::{fontstash::FontIx, FontTexture};

#[derive(Debug)]
pub struct FontBook {
    pub tex: FontTexture,
    pub store: FontStore,
}

#[derive(Debug, Default)]
pub struct FontStore {
    //
}

#[derive(Debug)]
pub struct FontFamily {
    pub regular: FontData,
    pub bold: Option<FontData>,
    pub italic: Option<FontData>,
}

#[derive(Debug)]
pub struct FontData {
    pub name: String,
    pub path: PathBuf,
    pub ix: FontIx,
}

#[derive(Debug)]
pub struct FontFamilyDesc {
    //
}

pub struct FontUse {
    pub family: FontFamily,
    pub style: FontStyle,
}

#[derive(Debug, Clone)]
pub struct FontStyle {
    pub font_ix: FontIx,
    pub fontsize: f32,
    pub line_spacing: f32,
    // pub is_bold: bool,
    // pub is_italic: bool,
    // pub shadow: TextShadowStyle
}
