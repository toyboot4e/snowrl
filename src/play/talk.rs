/*!
Talk
*/

use {rokol::fons::FontTexture, std::borrow::Cow};

use snow2d::{
    asset::AssetCacheAny,
    gfx::{
        geom2d::*,
        tex::{NineSliceSprite, SpriteData},
        text::style::FontStyle,
    },
    ui::node::*,
    utils::{arena::Index, ez, pool::Handle, tweak::*},
};

use grue2d::{
    rl::world::{actor::Actor, World},
    Global, UiLayer,
};

use crate::utils::{consts, paths};

/// Talk view description
#[derive(Debug)]
pub struct TalkViewCommand<'a> {
    pub txt: Cow<'a, str>,
    pub from: Index<Actor>,
    pub to: Index<Actor>,
    pub cfg: TalkConfig,
}

/// Direction and baloon kind
#[derive(Debug, Clone)]
pub struct TalkConfig {
    pub dir: TalkDirection,
    pub kind: TalkKind,
}

/// Up | Down
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TalkDirection {
    Up,
    Down,
}

/// Speak | Wonder
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TalkKind {
    Speak,
    Wonder,
}

/// Layout of talk window, text and baloon
#[derive(Debug)]
struct TalkLayout {
    pub txt: Vec2f,
    /// Center of the window
    pub win: Rect2f,
    /// Top-center of the baloon
    pub baloon: Vec2f,
}

impl<'a> TalkViewCommand<'a> {
    fn layout(
        &self,
        tcfg: &TalkConfig,
        fb: &FontTexture,
        fstyle: &FontStyle,
        world: &World,
    ) -> TalkLayout {
        let pos = Self::base_pos(world, self.to);
        self.layout_impl(tcfg, fb, fstyle, pos)
    }

    fn base_pos(world: &World, actor: Index<Actor>) -> Vec2f {
        let actor = &world.entities[actor];
        let mut pos = actor.img.pos_world_centered(&world.map.tiled);
        pos.y -= world.map.tiled.tile_height as f32;
        pos
    }

    fn layout_impl(
        &self,
        tcfg: &TalkConfig,
        fb: &FontTexture,
        fstyle: &FontStyle,
        pos: Vec2f,
    ) -> TalkLayout {
        let mut win_rect =
            fb.text_bounds_multiline(&self.txt, pos, fstyle.fontsize, fstyle.line_spacing);

        // FIXME: the hard-coded y alignment
        let mut baloon_pos = Vec2f::new(pos.x, pos.y + tweak!(11.0));

        // align horizontally the center of the window
        win_rect[0] -= win_rect[2] / 2.0;
        // align vertically the bottom of the window
        win_rect[1] -= win_rect[3];

        let mut txt_pos = Vec2f::new(win_rect[0], win_rect[1]);

        if tcfg.dir == TalkDirection::Down {
            // inverse horizontally
            baloon_pos.y += tweak!(59.0);
            txt_pos.y = pos.y + (pos.y - txt_pos.y);
            win_rect[1] = pos.y + (pos.y - win_rect[1]);
        }

        // add paddings
        win_rect[0] -= consts::TALK_PADS[0];
        win_rect[1] -= consts::TALK_PADS[1];
        win_rect[2] += consts::TALK_PADS[0] * 2.0;
        win_rect[3] += consts::TALK_PADS[1] * 2.0;

        TalkLayout {
            txt: txt_pos,
            win: win_rect.into(),
            baloon: baloon_pos,
        }
    }
}

/// Data to draw talk
#[derive(Debug)]
struct TalkView {
    layout: TalkLayout,
    win_sprite: NineSliceSprite,
    baloons: [SpriteData; 4],
}

impl TalkView {
    pub fn new(layout: TalkLayout, assets: &mut AssetCacheAny) -> Self {
        let win_sprite = NineSliceSprite {
            tex: assets.load_sync(paths::img::sourve::A).unwrap(),
        };

        let b = assets.load_sync(paths::img::sourve::BALOON).unwrap();
        let mut sprite = SpriteData::builder(b);
        sprite.origin([0.5, 0.5]);

        let baloons = [
            sprite.uv_rect([0.0, 0.0, 0.5, 0.5]).build(),
            sprite.uv_rect([0.5, 0.0, 0.5, 0.5]).build(),
            sprite.uv_rect([0.0, 0.5, 0.5, 0.5]).build(),
            sprite.uv_rect([0.5, 0.5, 0.5, 0.5]).build(),
        ];

        Self {
            layout,
            win_sprite,
            baloons,
        }
    }

    pub fn win(&self) -> &NineSliceSprite {
        &self.win_sprite
    }

    pub fn baloon(&self, cfg: &TalkConfig) -> &SpriteData {
        match (cfg.dir, cfg.kind) {
            (TalkDirection::Up, TalkKind::Speak) => &self.baloons[0],
            (TalkDirection::Down, TalkKind::Speak) => &self.baloons[1],
            (TalkDirection::Up, TalkKind::Wonder) => &self.baloons[2],
            (TalkDirection::Down, TalkKind::Wonder) => &self.baloons[3],
        }
    }
}

#[derive(Debug)]
struct TalkNodes {
    win: Handle<Node>,
    txt: Handle<Node>,
    baloon: Handle<Node>,
}

/// State to play talk
#[derive(Debug)]
pub struct PlayTalk {
    cfg: TalkConfig,
    view: TalkView,
    nodes: TalkNodes,
}

impl PlayTalk {
    pub fn new(talk: TalkViewCommand<'_>, gl: &mut Global) -> Self {
        // FIXME: use custom config
        let fstyle = FontStyle {
            font_ix: unsafe { snow2d::gfx::text::font::FontIx::from_raw(1) },
            fontsize: 20.0,
            // FIXME: layout only works with this spacing. why?
            line_spacing: 4.0,
        };
        let layout = talk.layout(&talk.cfg, &gl.ice.snow.fontbook.tex, &fstyle, &gl.world);
        let view = TalkView::new(layout, &mut gl.ice.assets);

        let layer = &mut gl.ui.get_mut(UiLayer::OnActors);
        let nodes = TalkNodes {
            win: layer.nodes.add({
                let mut win = Node::from(view.win());
                win.params.pos = view.layout.win.left_up().into();
                win
            }),
            txt: layer.nodes.add({
                let mut txt = Node::from(Text {
                    txt: talk.txt.into_owned(),
                });
                txt.params.pos = view.layout.txt.into();
                txt
            }),
            baloon: layer.nodes.add({
                let mut baloon = Node::from(view.baloon(&talk.cfg));
                baloon.params.pos = view.layout.baloon.into();
                baloon
            }),
        };

        let dt = ez::EasedDt::new(consts::TALK_WIN_ANIM_TIME, consts::TALK_WIN_EASE);
        let mut b = layer.anims.builder();

        let rect = &view.layout.win;
        b.dt(dt)
            .node(&nodes.win)
            .pos([(rect.x + rect.w / 2.0, rect.y), (rect.x, rect.y)])
            .size(([0.0, rect.h], rect.size()));
        b.node(&nodes.baloon).alpha([0, 255]);

        Self {
            cfg: talk.cfg,
            view,
            nodes,
        }
    }
}
