/*!
Renderer for the (not so) rich text `view`
*/

use crate::gfx::text::font::FontSetHandle;

/// How to draw a
#[derive(Debug)]
pub struct FontView {
    pub font: FontSetHandle,
    pub fontsize: u32,
}

#[derive(Debug)]
pub struct TextRenderConfig {
    pub line_spacing: u32,
}

#[derive(Debug)]
pub struct FontRenderState {
    pub default_font_view: FontView,
    pub default_cfg: TextRenderConfig,
}
