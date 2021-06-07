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

use std::{collections::HashMap, fs, path::Path};

use snow2d::{
    asset::Asset,
    gfx::{
        geom2d::Vec2f,
        tex::{pack, SpriteData, Texture2dDrop},
        text::{FontBook, FontFamilyHandle},
    },
    input::Key,
    prelude::Texture2d,
};

use self::{
    span::{FontFace, ParseError, Span, SpanLines},
    token::*,
    view::NodeLines,
};

/// Spritesheet of keyboard icons
#[derive(Debug)]
pub struct KbdIcons {
    tex: Asset<Texture2dDrop>,
    /// Hash map of uv rect (x, y, w, h)
    uvs: HashMap<Key, [f32; 4]>,
}

impl KbdIcons {
    pub fn new(tex: Asset<Texture2dDrop>, pack_json: &Path) -> anyhow::Result<Self> {
        let pack_text_bytes = fs::read(pack_json)?;
        let pack: pack::TexPack = serde_json::from_slice(&pack_text_bytes)?;

        let tex_size = tex.get().unwrap().sub_tex_size_scaled();

        let uvs = pack
            .frames
            .iter()
            .filter_map(|frame| {
                let uvs = {
                    let rect = &frame.frame;
                    [
                        rect.x as f32 / tex_size[0],
                        rect.y as f32 / tex_size[1],
                        rect.w as f32 / tex_size[0],
                        rect.h as f32 / tex_size[1],
                    ]
                };

                // (.*-)?([a-z]*)\.png
                //       ^------^ key_name
                let key_name = {
                    // ignore everything before a last `-`
                    let first = frame
                        .filename
                        .bytes()
                        .rposition(|b| b == b'-')
                        .map(|x| x + 1)
                        .unwrap_or(0);

                    // ignore everything after `.`
                    let dot = frame.filename.bytes().position(|b| b == b'.').unwrap();
                    &frame.filename[first..dot]
                };

                if key_name.len() == 1 {
                    let c = key_name.chars().next().unwrap();
                    Key::from_char(c).map(|key| (key, uvs))
                } else {
                    // TODO: support enter, F[0-12], space etc.
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        Ok(Self { tex, uvs })
    }

    pub fn get_sprite(&self, key: Key) -> Option<SpriteData> {
        self.uvs
            .get(&key)
            .map(|uv| SpriteData::builder(self.tex.clone()).uv_rect(*uv).build())
    }
}

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

/// Handle / lifetime of UI nodes created from markup text
#[derive(Debug)]
pub struct TextHandle {
    pub root: snow2d::ui::NodeHandle,
    /// Lifetime of chid nodes
    children: Vec<snow2d::ui::NodeHandle>,
}

#[derive(Debug)]
struct Text<'a> {
    nodes: NodeLines<'a>,
    // lifetime
    spans: SpanLines<'a>,
    tks: Vec<Token<'a>>,
}

/// Renders the simple markup langauge into snow2d UI nodes
pub struct Renderer<'a, 'b, 'c, 'd> {
    /// Fonts and font texture
    pub fb: &'a mut FontBook,
    /// Where we render UI nodes
    pub pool: &'b mut snow2d::ui::NodePool,
    /// Node configuration
    pub default_node: &'c snow2d::ui::Node,
    /// Spritesheet of keyboard icons
    pub kbd_icons: &'d mut KbdIcons,
}

/// Parses text into the rather-rich markup format
fn parse<'src>(
    src: &'src str,
    fb: &mut FontBook,
    cfg: &RenderConfig,
) -> Result<Text<'src>, ParseError> {
    let tks = token::tokenize(src)?;
    let spans = span::to_spans(&tks)?;
    let nodes = view::to_nodes(&spans, fb, cfg);

    Ok(Text { nodes, tks, spans })
}

impl<'a, 'b, 'c, 'd> Renderer<'a, 'b, 'c, 'd> {
    pub fn run(
        &mut self,
        cfg: &RenderConfig,
        pos: impl Into<Vec2f>,
        src: &str,
    ) -> Result<TextHandle, ParseError> {
        let text = self::parse(src, &mut self.fb, &cfg)?;
        let handle = self.render_text(cfg, &text);

        let root = &mut self.pool[&handle.root];
        root.params.pos = pos.into();

        Ok(handle)
    }

    /// Renders parsed text into `snow2d` ui node
    fn render_text<'src>(&mut self, cfg: &RenderConfig, text: &Text<'src>) -> TextHandle {
        use snow2d::ui::node;

        let mut y = 0.0;

        // TODO: use `.map` when Rust 2021 edition comes
        let mut lines = Vec::new();
        for ln in text.nodes.lines() {
            let mut line = Vec::new();

            for markup_node in ln {
                match &markup_node.sp {
                    Span::Text(text) => {
                        // FIXME: use user style
                        let mut ui_node = {
                            let mut ui_node =
                                node::Text::builder(text.slice.to_string(), &self.fb.tex);

                            ui_node.fontsize(cfg.fontsize);
                            ui_node.font(match text.font_face {
                                FontFace::Regular => cfg.font_family.regular(),
                                FontFace::Italic => cfg.font_family.italic(),
                                FontFace::Bold => cfg.font_family.bold(),
                            });
                            ui_node.build()
                        };

                        ui_node.layer = self.default_node.layer;
                        ui_node.z_order = self.default_node.z_order;

                        // TODO: measure height and align y
                        ui_node.params.pos = Vec2f::new(markup_node.geom.x, y);
                        ui_node.params.size = markup_node.geom.size;

                        line.push(ui_node);
                    }
                    Span::Image(_img) => {
                        todo!()
                    }
                    Span::Kbd(kbd) => {
                        for key in kbd.keys.iter().cloned() {
                            // TODO: return RenderError
                            let sprite = self.kbd_icons.get_sprite(key).unwrap_or_else(|| {
                                panic!("Unable to find sprite for key {:?}", key)
                            });

                            let mut ui_node = self.default_node.clone();
                            ui_node.surface = sprite.into();

                            ui_node.params.pos = Vec2f::new(markup_node.geom.x, y);
                            ui_node.params.size = markup_node.geom.size;

                            // algin vertically
                            ui_node.params.pos.y -= (markup_node.geom.size.y - cfg.fontsize) / 2.0;

                            line.push(ui_node);
                        }
                    }
                }
            }

            y += cfg.fontsize + cfg.nl_space;

            lines.push(line);
        }

        let root = self.pool.add({
            let mut node = self.default_node.clone();
            node.surface = node::Surface::None;
            node
        });

        let mut children = Vec::new();

        // hierarchy: root > line > node
        for line in lines {
            let line_node = self.pool.add_child(&root, {
                let mut node = self.default_node.clone();
                node.surface = node::Surface::None;
                node
            });

            for child in line {
                let child = self.pool.add_child(&line_node, child);
                children.push(child);
            }

            self.pool.attach_child(&root, &line_node);
        }

        TextHandle { root, children }
    }
}
