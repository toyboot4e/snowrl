/*!
Talk
*/

use {
    rokol::fons::{FontBook, FontConfig},
    std::borrow::Cow,
};

use snow2d::{
    asset::AssetCacheAny,
    gfx::{
        geom2d::*,
        tex::{NineSliceSprite, SpriteData},
    },
};

use rlbox::{
    ui::{node::*, Layer},
    utils::{arena::Index, ez, pool::Handle, tweak::*},
};

use grue2d::{
    rl::world::{actor::Actor, World},
    Global,
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
        fb: &FontBook,
        fcfg: &FontConfig,
        world: &World,
    ) -> TalkLayout {
        let pos = Self::base_pos(world, self.to);
        self.layout_impl(tcfg, fb, fcfg, pos)
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
        fb: &FontBook,
        fcfg: &FontConfig,
        pos: Vec2f,
    ) -> TalkLayout {
        let mut win_rect = fb.text_bounds(pos, fcfg, &self.txt);

        // FIXME: the hard-coded y alignment
        let mut baloon_pos = Vec2f::new(pos.x, pos.y + tweak!(11.0));

        // align horizontally the center of the window
        win_rect[0] -= win_rect[2] / 2.0;
        // align vertically the bottom of the window
        win_rect[1] -= win_rect[3];

        let mut txt_pos = Vec2f::new(win_rect[0], win_rect[1]);

        if tcfg.dir == TalkDirection::Down {
            // inverse horizontally
            baloon_pos.y += tweak!(61.0);
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

/// Renderer-agnostic talk surface
#[derive(Debug)]
struct TalkSurface {
    layout: TalkLayout,
    win_sprite: NineSliceSprite,
    baloons: [SpriteData; 4],
}

impl TalkSurface {
    pub fn new(layout: TalkLayout, assets: &mut AssetCacheAny) -> Self {
        let win_sprite = NineSliceSprite {
            tex: assets.load_sync(paths::img::sourve::A).unwrap(),
        };

        let b = assets.load_sync(paths::img::sourve::BALOON).unwrap();

        let baloons = [
            SpriteData {
                tex: b.clone(),
                uv_rect: [0.0, 0.0, 0.5, 0.5],
                // REMARK: we'll specify the center of the top-center of the baloon
                origin: [0.5, 0.5],
                ..Default::default()
            },
            SpriteData {
                tex: b.clone(),
                uv_rect: [0.5, 0.0, 0.5, 0.5],
                // REMARK: we'll specify the center of the top-center of the baloon
                origin: [0.5, 0.5],
                ..Default::default()
            },
            SpriteData {
                tex: b.clone(),
                uv_rect: [0.0, 0.5, 0.5, 0.5],
                // REMARK: we'll specify the center of the top-center of the baloon
                origin: [0.5, 0.5],
                ..Default::default()
            },
            SpriteData {
                tex: b.clone(),
                uv_rect: [0.5, 0.5, 0.5, 0.5],
                // REMARK: we'll specify the center of the top-center of the baloon
                origin: [0.5, 0.5],
                ..Default::default()
            },
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
    surface: TalkSurface,
    nodes: TalkNodes,
}

impl PlayTalk {
    pub fn new(talk: TalkViewCommand<'_>, gl: &mut Global, layer: Index<Layer>) -> Self {
        let layout = talk.layout(&talk.cfg, &gl.ice.rdr.fontbook, &gl.ice.font_cfg, &gl.world);
        let surface = TalkSurface::new(layout, &mut gl.ice.assets);

        let layer = &mut gl.ui.layers[layer];
        let nodes = TalkNodes {
            win: layer.nodes.add({
                let mut win = Node::from(surface.win());
                win.params.pos = surface.layout.win.left_up().into();
                win
            }),
            txt: layer.nodes.add({
                let mut txt = Node::from(Text {
                    txt: talk.txt.into_owned(),
                });
                txt.params.pos = surface.layout.txt.into();
                txt
            }),
            baloon: layer.nodes.add({
                let mut baloon = Node::from(surface.baloon(&talk.cfg));
                baloon.params.pos = surface.layout.baloon.into();
                baloon
            }),
        };

        let dt = ez::EasedDt::new(consts::TALK_WIN_ANIM_TIME, consts::TALK_WIN_EASE);
        let mut b = layer.anims.builder();

        let rect = &surface.layout.win;
        b.dt(dt)
            .node(&nodes.win)
            .pos([(rect.x + rect.w / 2.0, rect.y), (rect.x, rect.y)])
            .size(([0.0, rect.h], rect.size()));
        b.node(&nodes.baloon).alpha([0, 255]);

        Self {
            cfg: talk.cfg,
            surface,
            nodes,
        }
    }
}
