//! Frame-based actor sprite animation

use std::{collections::HashMap, time::Duration};

use snow2d::{
    asset::Asset,
    gfx::{
        draw::*,
        geom2d::*,
        tex::{SpriteData, Texture2dDrop},
    },
};

use crate::{
    render::anim::{FrameAnimPattern, FrameAnimState, LoopMode},
    rl::grid2d::*,
    utils::{ez, DoubleSwap},
};

/// Generate character walking animation with some heuristic
pub fn gen_anim_auto(
    tex: &Asset<Texture2dDrop>,
    fps: f32,
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    let size = tex.get().unwrap().sub_tex_size();
    if size[0] >= size[1] {
        self::gen_anim8(tex, fps)
    } else {
        self::gen_anim4(tex, fps)
    }
}

/// Generates character walking animation from 3x4 character image
pub fn gen_anim4(
    tex: &Asset<Texture2dDrop>,
    fps: f32,
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    self::gen_dir_anim_with(
        tex,
        fps,
        &DIR_4_ANIM_PATTERN,
        |ix| {
            let row = ix / 3;
            let col = ix % 3;
            // x, y, w, h
            [col as f32 / 3.0, row as f32 / 4.0, 1.0 / 3.0, 1.0 / 4.0]
        },
        |_sprite| {},
    )
}

/// Generates character walking animation from 6x4 character image
pub fn gen_anim8(
    tex: &Asset<Texture2dDrop>,
    fps: f32,
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    self::gen_dir_anim_with(
        tex,
        fps,
        &DIR_8_ANIM_PATTERN,
        |ix| {
            let row = ix / 6;
            let col = ix % 6;
            // x, y, w, h
            [col as f32 / 6.0, row as f32 / 4.0, 1.0 / 6.0, 1.0 / 4.0]
        },
        |_sprite| {},
    )
}

/// Maps [`Dir8`] to a diretional animation frame
type DirAnimPattern = [(Dir8, [usize; 3]); 8];

const DIR_4_ANIM_PATTERN: DirAnimPattern = [
    (Dir8::E, [6, 7, 8]),
    (Dir8::W, [3, 4, 5]),
    (Dir8::S, [0, 1, 2]),
    (Dir8::SE, [0, 1, 2]),
    (Dir8::SW, [0, 1, 2]),
    (Dir8::N, [9, 10, 11]),
    (Dir8::NE, [9, 10, 11]),
    (Dir8::NW, [9, 10, 11]),
];

const DIR_8_ANIM_PATTERN: DirAnimPattern = [
    (Dir8::E, [12, 13, 14]),
    (Dir8::W, [6, 7, 8]),
    (Dir8::S, [0, 1, 2]),
    (Dir8::SE, [9, 10, 11]),
    (Dir8::SW, [3, 4, 5]),
    (Dir8::N, [18, 19, 20]),
    (Dir8::NE, [21, 22, 23]),
    (Dir8::NW, [15, 16, 17]),
];

fn gen_dir_anim_with(
    tex: &Asset<Texture2dDrop>,
    fps: f32,
    patterns: &DirAnimPattern,
    gen_uv_rect: impl Fn(usize) -> [f32; 4],
    mut f: impl FnMut(&mut SpriteData),
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    patterns
        .iter()
        .map(|(dir, indices)| {
            (
                dir.clone(),
                FrameAnimPattern::new(
                    indices
                        .iter()
                        .map(|ix| {
                            let mut sprite = SpriteData {
                                tex: tex.clone(),
                                uv_rect: gen_uv_rect(*ix),
                                rot: 0.0,
                                // specify the center position of the image to place it
                                origin: [0.5, 0.5],
                                scales: [1.0, 1.0],
                            };

                            f(&mut sprite);

                            sprite
                        })
                        .collect::<Vec<_>>(),
                    fps,
                    LoopMode::PingPong,
                ),
            )
        })
        .collect()
}

/// An animatable actor image
#[derive(Debug, Clone)]
pub struct ActorImage {
    anim_state: FrameAnimState<Dir8, SpriteData>,
    state: DoubleSwap<ActorSnapshot>,
    /// Sec
    dt: ez::EasedDt,
}

/// Interpolate two snapshots to draw actor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSnapshot {
    pos: Vec2i,
    dir: Dir8,
}

impl ActorImage {
    pub fn new(
        tex: Asset<Texture2dDrop>,
        anim_fps: f32,
        walk_secs: f32,
        walk_ease: ez::Ease,
        pos: Vec2i,
        dir: Dir8,
    ) -> snow2d::gfx::tex::Result<Self> {
        let anim = self::gen_anim_auto(&tex, anim_fps);

        let data = ActorSnapshot { pos, dir };

        Ok(Self {
            anim_state: FrameAnimState::new(anim, dir),
            state: DoubleSwap::new(data, data),
            dt: ez::EasedDt::new(walk_secs, walk_ease),
        })
    }

    /// Sets position and direction without animation
    pub fn warp(&mut self, pos: Vec2i, dir: Dir8) {
        let next_snap = ActorSnapshot { dir, pos };
        self.state.set_a(next_snap);
        self.state.set_b(next_snap);
        self.anim_state.set_pattern(dir, true);
    }

    /// Updates the image with (new) actor position and direction
    pub fn update(&mut self, dt: Duration, pos: Vec2i, dir: Dir8) {
        if dir != self.state.a().dir {
            self.anim_state.set_pattern(dir, false);
        }

        // update interpolation value
        if pos != self.state.a().pos {
            self.dt.reset();
        }
        self.dt.tick(dt);

        let next_snap = ActorSnapshot { dir, pos };
        if next_snap != *self.state.a() {
            self.state.swap();
            self.state.set_a(next_snap);
        }

        // update animation frame
        self.anim_state.tick(dt);
    }

    pub fn pos_screen(&self, tiled: &tiled::Map) -> Vec2f {
        let pos_prev = self.align(self.state.b().pos, tiled);
        let pos_curr = self.align(self.state.a().pos, tiled);

        pos_prev * (1.0 - self.dt.get()) + pos_curr * self.dt.get()
    }

    pub fn render<'a, 'b, D: DrawApi>(
        &'a self,
        draw: &'b mut D,
        tiled: &tiled::Map,
    ) -> impl QuadParamsBuilder + 'a
    where
        'b: 'a,
    {
        let pos = self.pos_screen(tiled);

        let mut x = draw.sprite(self.sprite());
        x.dst_pos_px(pos);
        x
    }

    /// Align the bottom-center of an actor to the bottom-center of a cell
    fn align(&self, pos: Vec2i, tiled: &tiled::Map) -> Vec2f {
        let mut pos = crate::render::tiled::t2w_center(pos, &tiled);
        pos.y += tiled.tile_height as f32 / 2.0;
        pos.y -= self.sprite().sub_tex_size()[1] / 2.0;
        pos
    }

    /// Sprite for current frame
    pub fn sprite(&self) -> &SpriteData {
        self.anim_state.current_frame()
    }
}
