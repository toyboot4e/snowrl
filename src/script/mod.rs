//! Script

use {
    rokol::fons::{FontBook, FontConfig},
    snow2d::gfx::geom2d::*,
};

use crate::{turn::tick::ActorIx, world::World};

// pub trait Script {
// }

// pub trait Interactable {
//     fn on_interact(&mut self) -> Option<Box<dyn Script>>;
// }

#[derive(Debug, Clone, Copy)]
pub enum ScriptRef {
    Interact { from: ActorIx, to: ActorIx },
}

// Generates [`TalkLayout`]
pub struct Talk<'a> {
    pub txt: &'a str,
    pub from: ActorIx,
    pub to: ActorIx,
}

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
