//! Script

use {
    rlbox::utils::ez,
    rokol::fons::{FontBook, FontConfig},
    snow2d::gfx::{
        draw::*,
        geom2d::*,
        tex::{NineSliceSprite, SpriteData},
        Color, RenderPass,
    },
    std::{borrow::Cow, time::Duration},
};

use crate::{fsm::Global, turn::tick::ActorIx, world::World};

// pub trait Script {
// }

// pub trait Interactable {
//     fn on_interact(&mut self) -> Option<Box<dyn Script>>;
// }

#[derive(Debug, Clone, Copy)]
pub enum ScriptRef {
    Interact { from: ActorIx, to: ActorIx },
}

/// Generates [`TalkLayout`]
#[derive(Debug)]
pub struct Talk<'a> {
    pub txt: Cow<'a, str>,
    pub from: ActorIx,
    pub to: ActorIx,
}

/// Play text and wait for it
#[derive(Debug)]
pub struct PlayTalk {
    window: NineSliceSprite,
    baloon: SpriteData,
    layout: TalkLayout,
    dt_win: ez::EasedDt,
    txt: PlayText,
}

impl PlayTalk {
    pub fn new(gl: &mut Global, talk: Talk<'_>) -> Self {
        let window = NineSliceSprite {
            tex: gl
                .wcx
                .assets
                .load_sync(crate::paths::img::sourve::A)
                .unwrap(),
        };

        let baloon = SpriteData {
            tex: gl
                .wcx
                .assets
                .load_sync(crate::paths::img::sourve::BALOON)
                .unwrap(),
            uv_rect: [0.0, 0.0, 0.5, 0.5],
            // REMARK: we'll specify the center of the top-center of the baloon
            origin: [0.5, 0.0],
            ..Default::default()
        };

        let layout = talk.layout(&gl.wcx.rdr.fontbook, &gl.wcx.font_cfg, &gl.world);

        // count visible characters
        let n_chars = talk
            .txt
            .chars()
            .filter(|c| !matches!(*c, '\n' | '\t'))
            .count();

        let txt = PlayText::new(talk.txt, layout.txt);

        Self {
            window,
            baloon,
            layout,
            dt_win: ez::EasedDt::new(
                crate::consts::TALK_WIN_ANIM_TIME,
                crate::consts::TALK_WIN_EASE,
            ),
            txt,
        }
    }

    /// Initializes `self` for next text play
    pub fn init(&mut self, gl: &mut Global, talk: Talk<'_>) {
        self.layout = talk.layout(&gl.wcx.rdr.fontbook, &gl.wcx.font_cfg, &gl.world);
        self.txt.init(talk.txt, self.layout.txt);
    }

    pub fn update(&mut self, dt: Duration) {
        self.txt.update(dt);
        self.dt_win.tick(dt);
    }

    pub fn render(&mut self, screen: &mut RenderPass<'_>) {
        // consider tween
        let t = self.dt_win.get();

        let rect = &self.layout.win;
        screen.sprite(&self.window).dst_rect_px([
            rect.x + rect.w * (1.0 - t) / 2.0,
            rect.y + rect.h * (1.0 - t) / 2.0,
            rect.w * t,
            rect.h * t,
        ]);

        self.txt.render(screen);

        // baloon
        let color = Color::WHITE.with_alpha((255.0 * t) as u8);
        screen
            .sprite(&self.baloon)
            .dst_pos_px(self.layout.baloon)
            .color(color);
    }
}

/// Layout of talk window, text and baloon
#[derive(Debug)]
pub struct TalkLayout {
    pub txt: Vec2f,
    /// Center of the window
    pub win: Rect2f,
    /// Top-center of the baloon
    pub baloon: Vec2f,
}

impl<'a> Talk<'a> {
    fn base_pos(world: &World, actor: ActorIx) -> Vec2f {
        let actor = &world.entities[actor.0];
        let mut pos = actor.img.pos_screen(&world.map.tiled);
        pos.y -= world.map.tiled.tile_height as f32;
        pos
    }

    /// Layout talk window, text and baloon
    ///
    /// * `pos`: center of a cell
    pub fn layout(&self, fb: &FontBook, fcfg: &FontConfig, world: &World) -> TalkLayout {
        let pos = Self::base_pos(world, self.to);
        self.layout_impl(fb, fcfg, pos)
    }

    fn layout_impl(&self, fb: &FontBook, fcfg: &FontConfig, pos: Vec2f) -> TalkLayout {
        let mut win_rect = fb.text_bounds(pos, fcfg, &self.txt);

        // FIXME: the hard-coded y alignment
        let baloon_pos = Vec2f::new(win_rect[0], win_rect[1] - 24.0);

        // align horizontally the center of the window
        win_rect[0] -= win_rect[2] / 2.0;
        // align vertically the bottom of the window
        win_rect[1] -= win_rect[3];

        let txt_pos = Vec2f::new(win_rect[0], win_rect[1]);

        // add paddings
        win_rect[0] -= crate::consts::TALK_PADS[0];
        win_rect[1] -= crate::consts::TALK_PADS[1];
        win_rect[2] += crate::consts::TALK_PADS[0] * 2.0;
        win_rect[3] += crate::consts::TALK_PADS[1] * 2.0;

        TalkLayout {
            txt: txt_pos,
            win: win_rect.into(),
            baloon: baloon_pos,
        }
    }
}

#[derive(Debug, Default)]
pub struct PlayText {
    pub txt: String,
    pub n_chars: usize,
    pub pos: Vec2f,
    pub dt: ez::NormDt,
}

impl PlayText {
    pub fn new(txt: impl Into<String>, pos: Vec2f) -> Self {
        let mut me = Self::default();
        me.init(txt, pos);
        me
    }

    /// Initializes `self` for next text play
    pub fn init(&mut self, txt: impl Into<String>, pos: Vec2f) {
        self.txt = txt.into();
        self.pos = pos;

        // count visible characters
        self.n_chars = self
            .txt
            .chars()
            .filter(|c| !matches!(*c, '\n' | '\t'))
            .count();

        self.dt = ez::NormDt::new(self.n_chars as f32 / crate::consts::CHARS_PER_SEC);
    }

    pub fn update(&mut self, dt: Duration) {
        self.dt.tick(dt);
    }

    pub fn render(&mut self, screen: &mut RenderPass<'_>) {
        // consider tween
        // let t = self.dt.get();

        // let rect = [
        //     self.rect.x + self.rect.w * (1.0 - t) / 2.0,
        //     self.rect.y + self.rect.h * (1.0 - t) / 2.0,
        //     self.rect.w * t,
        //     self.rect.h * t,
        // ];

        // let color = Color::WHITE.with_alpha((255.0 * t) as u8);

        // TODO: animate texts
        screen.txt(self.pos, &self.txt);
    }
}
