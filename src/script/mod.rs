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
pub struct PlayText {
    window: NineSliceSprite,
    baloon: SpriteData,
    layout: TalkLayout,
    dt_txt: ez::NormDt,
    dt_win: ez::EasedDt,
    txt: String,
    n_chars: usize,
}

impl PlayText {
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
            // REMARK: we'll specify the center of the top edge of the baloon
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

        Self {
            window,
            baloon,
            layout,
            n_chars,
            dt_txt: ez::NormDt::new(n_chars as f32 / crate::consts::CHARS_PER_SEC),
            dt_win: ez::EasedDt::new(
                crate::consts::TALK_WIN_ANIM_TIME,
                crate::consts::TALK_WIN_EASE,
            ),
            txt: talk.txt.into_owned(),
        }
    }

    /// Initializes `self` for next text play
    pub fn init(&mut self, gl: &mut Global, talk: Talk<'_>) {
        self.layout = talk.layout(&gl.wcx.rdr.fontbook, &gl.wcx.font_cfg, &gl.world);
        self.dt_txt.reset();
    }

    pub fn update(&mut self, dt: Duration) {
        self.dt_txt.tick(dt);
        self.dt_win.tick(dt);
    }

    pub fn render(&mut self, screen: &mut RenderPass<'_>) {
        // consider tween
        let t = self.dt_win.get();

        let rect = &self.layout.win_rect_center;
        screen.sprite(&self.window).dst_rect_px([
            rect.x + rect.w * (1.0 - t) / 2.0,
            rect.y + rect.h * (1.0 - t) / 2.0,
            rect.w * t,
            rect.h * t,
        ]);

        let color = Color::WHITE.with_alpha((255.0 * t) as u8);

        // TODO: animate texts
        screen.txt(self.layout.txt, &self.txt);

        // baloon
        screen
            .sprite(&self.baloon)
            .dst_pos_px(self.layout.baloon_center)
            .color(color);
    }
}

/// Layout of talk window, text and baloon
#[derive(Debug)]
pub struct TalkLayout {
    pub txt: Vec2f,
    /// TODO: Position is the center of the window
    pub win_rect_center: Rect2f,
    pub baloon_center: Vec2f,
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
            win_rect_center: win_rect.into(),
            baloon_center: baloon_pos,
        }
    }
}
