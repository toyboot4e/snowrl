/*!
Spans with semantic information and geometric information
*/

use snow2d::gfx::{geom2d::Vec2f, text::FontBook};

use crate::markup::{
    span::{LineSpan, Span, SpanLines},
    RenderConfig,
};

/// `SpanLines` -> `NodeLines`
pub fn to_nodes<'a>(spans: &SpanLines<'a>, fb: &mut FontBook, cfg: &RenderConfig) -> NodeLines<'a> {
    let mut nodes = vec![];

    for ln in spans.lines() {
        let mut offset = 0.0;

        for sp in ln {
            let size = self::measure(sp, fb, cfg);

            nodes.push(Node {
                sp: sp.clone(),
                geom: Geom { x: offset, size },
            });

            offset += size.x;
        }
    }

    NodeLines {
        nodes,
        // TODO: don't clone
        lines: spans.line_spans().to_vec(),
    }
}

/// Geometry data of rich text [`Node`]
#[derive(Debug, Clone, PartialEq)]
pub struct Geom {
    pub x: f32,
    pub size: Vec2f,
}

/// List of [`Node`] that can be split into lines
#[derive(Debug)]
pub struct NodeLines<'a> {
    nodes: Vec<Node<'a>>,
    lines: Vec<LineSpan>,
}

impl<'a> NodeLines<'a> {
    pub fn lines(&self) -> impl Iterator<Item = &[Node<'a>]> {
        self.lines
            .iter()
            .cloned()
            .map(move |ln| &self.nodes[ln.lo..ln.hi])
    }
}

/// Rich span with geometry information
#[derive(Debug)]
pub struct Node<'a> {
    pub sp: Span<'a>,
    pub geom: Geom,
}

fn measure(sp: &Span, fb: &FontBook, cfg: &RenderConfig) -> Vec2f {
    match sp {
        Span::Text(text) => {
            fb.tex.set_size(cfg.fontsize);
            Vec2f::from(fb.tex.text_size_oneline(text.slice))
        }
        Span::Image(_img) => {
            todo!()
        }
        // TODO: don't use heuristic. measure size using keyboard icons
        Span::Kbd(kbd) => Vec2f::new(kbd.len() as f32 * cfg.fontsize, cfg.fontsize),
    }
}
