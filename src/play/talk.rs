//! Talk

use {
    rlbox::utils::ez,
    rokol::fons::{FontBook, FontConfig},
    std::{borrow::Cow, time::Duration},
};

use snow2d::{
    asset::AssetCacheAny,
    gfx::{
        draw::*,
        geom2d::*,
        tex::{NineSliceSprite, SpriteData},
        Color, RenderPass,
    },
    Ice,
};

use crate::{
    play::txt::PlayText,
    turn::tick::ActorIx,
    utils::{consts, paths},
    world::World,
};

/// Talk command description
#[derive(Debug)]
pub struct TalkCommand<'a> {
    pub txt: Cow<'a, str>,
    pub from: ActorIx,
    pub to: ActorIx,
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

impl<'a> TalkCommand<'a> {
    fn layout(
        &self,
        tcfg: &TalkConfig,
        fb: &FontBook,
        fcfg: &FontConfig,
        world: &World,
    ) -> TalkLayout {
        println!("X");
        println!("IMPL: {:?}", tcfg);
        let pos = Self::base_pos(world, self.to);
        self.layout_impl(tcfg, fb, fcfg, pos)
    }

    fn base_pos(world: &World, actor: ActorIx) -> Vec2f {
        let actor = &world.entities[actor.0];
        let mut pos = actor.img.pos_screen(&world.map.tiled);
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
        let mut baloon_pos = Vec2f::new(win_rect[0], win_rect[1] + 8.0);

        // align horizontally the center of the window
        win_rect[0] -= win_rect[2] / 2.0;
        // align vertically the bottom of the window
        win_rect[1] -= win_rect[3];

        let mut txt_pos = Vec2f::new(win_rect[0], win_rect[1]);

        if tcfg.dir == TalkDirection::Down {
            // inverse horizontally
            baloon_pos.y = pos.y + (pos.y - baloon_pos.y) + 96.0 - 11.0;
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

/// State to play talk
#[derive(Debug)]
pub struct PlayTalk {
    cfg: TalkConfig,
    surface: TalkSurface,
    dt_win: ez::EasedDt,
    txt: PlayText,
}

impl PlayTalk {
    pub fn new(talk: TalkCommand<'_>, ice: &mut Ice, world: &World) -> Self {
        let layout = talk.layout(&talk.cfg, &ice.rdr.fontbook, &ice.font_cfg, world);
        let txt = PlayText::new(talk.txt, layout.txt);
        let surface = TalkSurface::new(layout, &mut ice.assets);

        Self {
            cfg: talk.cfg,
            surface,
            txt,
            dt_win: ez::EasedDt::new(consts::TALK_WIN_ANIM_TIME, consts::TALK_WIN_EASE),
        }
    }

    // /// Initializes `self` for next text play
    // pub fn init(&mut self, gl: &mut Global, talk: TalkCommand<'_>) {
    //     self.surface.layout = talk.layout(&gl.ice.rdr.fontbook, &gl.ice.font_cfg, &gl.world);
    //     self.txt.init(talk.txt, self.surface.layout.txt);
    // }

    pub fn update(&mut self, dt: Duration) {
        self.txt.update(dt);
        self.dt_win.tick(dt);
    }

    pub fn render(&mut self, screen: &mut RenderPass<'_>) {
        // consider tween
        let t = self.dt_win.get();

        // TODO: refactor with more generic tween
        let rect = &self.surface.layout.win;
        screen.sprite(self.surface.win()).dst_rect_px([
            // because our widnwo is aligned to left-up,
            // we manually align our window to the center
            rect.x + rect.w * (1.0 - t) / 2.0,
            rect.y,
            rect.w * t,
            rect.h,
        ]);

        self.txt.render(screen);

        // baloon
        let color = Color::WHITE.with_alpha((255.0 * t) as u8);
        screen
            .sprite(self.surface.baloon(&self.cfg))
            .dst_pos_px(self.surface.layout.baloon)
            .color(color);
    }
}
