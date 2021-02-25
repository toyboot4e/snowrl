/*!
Style of text span
*/

use rokol::fons::fontstash::FontIx;

use crate::gfx::geom2d::Vec2f;

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: [u8; 4],
    pub is_bold: bool,
    // TODO: use `FontStyle`
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
