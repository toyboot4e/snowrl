/*!
Style of text span used by `view`
*/

pub use rokol::fons::fontstash::FontIx;

use crate::gfx::geom2d::Vec2f;

#[derive(Debug, Clone)]
pub struct FontStyle {
    pub font_ix: FontIx,
    pub fontsize: f32,
    pub line_spacing: f32,
    // pub is_bold: bool,
    // pub is_italic: bool,
    // pub shadow: TextShadowStyle
}

// pub struct FontUse {
//     pub family: FontFamily,
//     pub style: FontStyle,
// }

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: [u8; 4],
    pub is_bold: bool,
    // TODO: use `FontStyle` alternative?
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            is_bold: false,
            color: [255, 255, 255, 255],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextShadowStyle {
    pub offset: Option<Vec2f>,
    pub color: [u8; 8],
}
