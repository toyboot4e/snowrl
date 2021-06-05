/*!
Talk
*/

use {rokol::fons::FontTexture, std::borrow::Cow};

use snow2d::{
    asset::AssetCache,
    gfx::{
        geom2d::*,
        tex::{NineSliceSprite, SpriteData},
        text::font::FontIx,
    },
    ui::{anim_builder::AnimGen, node::*, Node},
    utils::{arena::Index, ez, pool::Handle, tweak::*},
};

use grue2d::game::{
    data::{
        res::UiLayer,
        world::{actor::Actor, World},
    },
    Data,
};

use crate::utils::{consts, paths};

/// TODO: move it somewhere else
#[derive(Debug, Clone)]
pub struct FontStyle {
    pub font_ix: FontIx,
    pub fontsize: f32,
    pub ln_space: f32,
}

/// Talk view description
#[derive(Debug)]
pub struct TalkViewCommand<'a> {
    pub txt: Cow<'a, str>,
    pub from: Index<Actor>,
    pub to: Index<Actor>,
    pub cfg: TalkConfig,
}

/// Direction and baloon kind
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, PartialEq)]
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
        self.layout_impl(tcfg, fb, fstyle, pos, world)
    }

    fn base_pos(world: &World, actor: Index<Actor>) -> Vec2f {
        let actor = &world.entities[actor];
        let mut pos = actor.view.pos_world_centered(&world.map.tiled);
        pos.y -= world.map.tiled.tile_height as f32;
        pos
    }

    fn layout_impl(
        &self,
        tcfg: &TalkConfig,
        fb: &FontTexture,
        fstyle: &FontStyle,
        // center of cell the entity of is talked from
        pos: Vec2f,
        world: &World,
    ) -> TalkLayout {
        let mut win_rect =
            fb.text_bounds_multiline(&self.txt, pos, fstyle.fontsize, fstyle.ln_space);

        // FIXME:

        // FIXME: the hard-coded y alignment
        let mut baloon_pos = Vec2f::new(pos.x, pos.y + tweak!(11.0));

        // align horizontally the center of the window
        win_rect[0] -= win_rect[2] / 2.0;
        // align vertically the bottom of the window
        win_rect[1] -= win_rect[3];

        let mut txt_pos = Vec2f::new(win_rect[0], win_rect[1]);

        if tcfg.dir == TalkDirection::Down {
            let h = world.map.tiled.tile_height as f32;
            // only ballon has origin at [0.5, 0.5]
            baloon_pos.y += h * 1.5;
            txt_pos.y += win_rect[3] + h * 2.0;
            win_rect[1] += win_rect[3] + h * 2.0;
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
#[derive(Debug, PartialEq)]
struct TalkView {
    layout: TalkLayout,
    win_sprite: NineSliceSprite,
    baloons: [SpriteData; 4],
}

impl TalkView {
    pub fn new(layout: TalkLayout, assets: &mut AssetCache) -> Self {
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

#[derive(Debug, PartialEq)]
struct TalkNodes {
    win: Handle<Node>,
    txt: Handle<Node>,
    baloon: Handle<Node>,
}

/// State to play talk
#[derive(Debug, PartialEq)]
pub struct PlayTalk {
    cfg: TalkConfig,
    view: TalkView,
    nodes: TalkNodes,
}

impl PlayTalk {
    pub fn new(talk: TalkViewCommand<'_>, data: &mut Data) -> Self {
        // FIXME: use custom config
        let fstyle = FontStyle {
            font_ix: unsafe { snow2d::gfx::text::font::FontIx::from_raw(1) },
            fontsize: 22.0,
            ln_space: 2.0,
        };

        let layout = talk.layout(&talk.cfg, &data.ice.snow.fontbook.tex, &fstyle, &data.world);
        let view = TalkView::new(layout, &mut data.ice.assets);

        let ui = &mut data.res.ui;
        let nodes = TalkNodes {
            win: ui.nodes.add({
                let mut win = Node::from(view.win());
                win.layer = UiLayer::OnShadow.to_layer();
                win.params.pos = view.layout.win.left_up().into();
                win
            }),
            txt: ui.nodes.add({
                let mut txt = {
                    let mut txt = Text::builder(talk.txt.into_owned(), &data.ice.snow.fontbook.tex);
                    txt.fontsize(fstyle.fontsize).ln_space(fstyle.ln_space);
                    txt.build()
                };

                txt.layer = UiLayer::OnShadow.to_layer();
                txt.params.pos = view.layout.txt.into();
                txt
            }),
            baloon: ui.nodes.add({
                let mut baloon = Node::from(view.baloon(&talk.cfg));
                baloon.layer = UiLayer::OnShadow.to_layer();
                baloon.params.pos = view.layout.baloon.into();
                baloon
            }),
        };

        let dt = ez::EasedDt::new(consts::TALK_WIN_ANIM_TIME, consts::TALK_WIN_EASE);

        let rect = &view.layout.win;
        let mut gen = AnimGen::default();
        gen.dt(dt).node(&nodes.win);
        ui.anims
            .insert(gen.pos([(rect.x + rect.w / 2.0, rect.y), (rect.x, rect.y)]));
        ui.anims.insert(gen.size(([0.0, rect.h], rect.size())));
        ui.anims.insert(gen.node(&nodes.baloon).alpha([0, 255]));

        Self {
            cfg: talk.cfg,
            view,
            nodes,
        }
    }
}
