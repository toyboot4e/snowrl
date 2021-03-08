/*!
Representation of (not so) rich text view
*/

use super::style::*;

pub type QuadIx = u32;

/// Multiple lines of text
#[derive(Debug, Clone)]
pub struct TextView<'a> {
    pub text: &'a str,
    pub lines: Vec<LineView<'a>>,
}

/// One line of text
#[derive(Debug, Clone)]
pub struct LineView<'a> {
    pub line_spans: Vec<LineSpanView<'a>>,
}

/// Span of styled text
#[derive(Debug, Clone)]
pub struct LineSpanView<'a> {
    pub text_slice: &'a str,
    /// First quad index
    pub first_quad_ix: QuadIx,
    /// Quad span `[frm, to)`; relative to the first quad index
    pub quad_span: QuadSpan,
    /// Font and color
    pub style: TextStyle,
}

impl<'a> LineSpanView<'a> {
    pub fn quad_indices(&self) -> std::ops::Range<QuadIx> {
        (self.first_quad_ix + self.quad_span.from)..(self.first_quad_ix + self.quad_span.to)
    }
}

/// `[from, to)`; same as "text"[from..to]
#[derive(Debug, Clone)]
pub struct QuadSpan {
    pub from: QuadIx,
    pub to: QuadIx,
}
