/*!
Simple markup language integration

Use [`Renderer`] to create UI nodes from the annonymous markup language.

# Conversion steps

Markup text → [`token`] s → [`span`] s → [`view`] nodes → UI nodes in `snow2d`

# Modules

* [`token`] contains grammer.
* [`span`] contains semantics.
* [`view`] contains layout logic.
*/

pub mod span;
pub mod token;
pub mod view;

use snow2d::gfx::{
    geom2d::Vec2f,
    text::{FontBook, FontFamilyHandle},
};

use self::{
    span::{FontFace, ParseError, Span, SpanLines},
    token::*,
    view::NodeLines,
};

/// Configuration to render markup text into UI nodes
#[derive(Debug)]
pub struct RenderConfig {
    /// Default font family
    pub font_family: FontFamilyHandle,
    /// Default font size
    pub fontsize: f32,
    /// Default space between lines
    pub nl_space: f32,
}

/// Binding of context to [`run`](Self::run) the rendering method
pub struct Renderer<'a, 'b, 'c, 'd> {
    /// Fonts and font texture
    pub fb: &'a mut FontBook,
    /// The rendering configuration
    pub cfg: &'b RenderConfig,
    /// Where we render UI nodes
    pub pool: &'c mut snow2d::ui::NodePool,
    /// Node configuration
    pub default_node: &'d snow2d::ui::Node,
}

impl<'a, 'b, 'c, 'd> Renderer<'a, 'b, 'c, 'd> {
    pub fn run(self, src: &str) -> Result<TextHandle, ParseError> {
        self::render(src, self.fb, self.cfg, self.pool, self.default_node)
    }
}

/// Handle / lifetime of UI nodes created from markup text
#[derive(Debug)]
pub struct TextHandle {
    root: snow2d::ui::NodeHandle,
    /// Lifetime of chid nodes
    children: Vec<snow2d::ui::NodeHandle>,
}

/// Parses markup text and renders it into UI nodes
fn render<'a>(
    src: &'a str,
    fb: &mut FontBook,
    cfg: &RenderConfig,
    pool: &mut snow2d::ui::NodePool,
    default_node: &snow2d::ui::Node,
) -> Result<TextHandle, ParseError> {
    let text = self::parse(src, fb, cfg)?;
    Ok(self::render_text(&text, fb, cfg, pool, default_node))
}

#[derive(Debug)]
struct Text<'a> {
    nodes: NodeLines<'a>,
    // lifetime
    spans: SpanLines<'a>,
    tks: Vec<Token<'a>>,
}

/// Parses text into the rather-rich markup format
fn parse<'a>(src: &'a str, fb: &mut FontBook, cfg: &RenderConfig) -> Result<Text<'a>, ParseError> {
    let tks = token::tokenize(src)?;
    let spans = span::to_spans(&tks)?;
    let nodes = view::to_nodes(&spans, fb, cfg);

    Ok(Text { nodes, tks, spans })
}

/// Renders parsed text into `snow2d` ui node
fn render_text<'a>(
    text: &Text<'a>,
    fb: &mut FontBook,
    cfg: &RenderConfig,
    pool: &mut snow2d::ui::NodePool,
    default_node: &snow2d::ui::Node,
) -> TextHandle {
    use snow2d::ui::{node, Node};

    let mut y = 0.0;
    let lines = text.nodes.lines().map(|ln| {
        let mut line = Vec::new();

        for markup_node in ln {
            let ui_node = match &markup_node.sp {
                Span::Text(text) => {
                    // FIXME: use user style
                    let mut ui_node = node::Text::builder(text.slice.to_string(), &fb.tex);

                    ui_node.fontsize(cfg.fontsize);
                    ui_node.font(match text.font_face {
                        FontFace::Regular => cfg.font_family.regular(),
                        FontFace::Italic => cfg.font_family.italic(),
                        FontFace::Bold => cfg.font_family.bold(),
                    });

                    let mut ui_node = ui_node.build();
                    ui_node.layer = default_node.layer;
                    ui_node.z_order = default_node.z_order;

                    // TODO: measure height and align y
                    ui_node.params.pos = Vec2f::new(markup_node.geom.x, y);
                    ui_node.params.size = markup_node.geom.size;

                    ui_node
                }
                Span::Image(_img) => {
                    todo!()
                }
            };

            line.push(ui_node);
        }

        y += cfg.fontsize + cfg.nl_space;

        line
    });

    let root = pool.add({
        let mut node = default_node.clone();
        node.surface = node::Surface::None;
        node
    });

    let mut children = Vec::new();

    for line in lines {
        let parent = pool.add({
            let mut node = default_node.clone();
            node.surface = node::Surface::None;
            node
        });

        for child in line {
            let child = pool.add_child(&parent, child);
            children.push(child);
        }

        pool.attach_child(&root, &parent);
    }

    TextHandle { root, children }
}
