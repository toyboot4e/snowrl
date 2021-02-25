/*!
Layout of styled text
*/

use rokol::fons::{fontstash::FontIx, FonsQuad};

use super::style::*;

pub type QuadIx = u32;

/// Multiple lines of text
#[derive(Debug, Clone)]
pub struct TextLayout<'a> {
    pub text: &'a str,
    pub lines: Vec<LineLayout<'a>>,
}

/// One line of text
#[derive(Debug, Clone)]
pub struct LineLayout<'a> {
    pub line_spans: Vec<LineLayoutSpan<'a>>,
}

/// Span of styled text
#[derive(Debug, Clone)]
pub struct LineLayoutSpan<'a> {
    pub text_slice: &'a str,
    /// First quad index in the [`TextLayout`]
    pub first_quad_ix: QuadIx,
    /// [frm, to); relative to the first quad of the line
    pub quad_span: QuadSpan,
    pub style: TextStyle,
}

impl<'a> LineLayoutSpan<'a> {
    pub fn quad_indices(&self) -> std::ops::Range<QuadIx> {
        (self.first_quad_ix + self.quad_span.from)..(self.first_quad_ix + self.quad_span.to)
    }
}

/// [from, to); same as "text"[from..to]
#[derive(Debug, Clone)]
pub struct QuadSpan {
    pub from: QuadIx,
    pub to: QuadIx,
}
