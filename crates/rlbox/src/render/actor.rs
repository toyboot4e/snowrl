//! Frame-based actor sprite animation

use std::{collections::HashMap, path::Path, time::Duration};

use snow2d::gfx::{
    batcher::draw::*,
    geom2d::*,
    tex::{SharedSubTexture2d, SharedTexture2d, SpriteData, Texture2dDrop},
};

use crate::{
    render::anim::{FrameAnimPattern, FrameAnimState, LoopMode},
    rl::grid2d::*,
    utils::DoubleSwap,
};

/// Generates character walking animation from 4x3 character image
pub fn gen_anim4(
    texture: &SharedTexture2d,
    fps: f32,
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    self::gen_anim4_with(texture, fps, |_sprite| {})
}

/// Generates character walking animation from 4x3 character image, letting user modify the sprite
pub fn gen_anim4_with(
    texture: &SharedTexture2d,
    fps: f32,
    mut f: impl FnMut(&mut SpriteData),
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    [
        (Dir8::E, [6, 7, 8]),
        (Dir8::W, [3, 4, 5]),
        (Dir8::S, [0, 1, 2]),
        (Dir8::SE, [0, 1, 2]),
        (Dir8::SW, [0, 1, 2]),
        (Dir8::N, [9, 10, 11]),
        (Dir8::NE, [9, 10, 11]),
        (Dir8::NW, [9, 10, 11]),
    ]
    .iter()
    .map(|(dir, ixs)| {
        (
            dir.clone(),
            FrameAnimPattern::new(
                ixs.iter()
                    .map(|ix| {
                        let row = ix / 3;
                        let col = ix % 3;
                        let uv_pos = [col as f32 / 3.0, row as f32 / 4.0];
                        let uv_size = [1.0 / 3.0, 1.0 / 4.0];

                        let mut sprite = SpriteData {
                            sub_tex: SharedSubTexture2d {
                                shared: texture.clone(),
                                uv_rect: [uv_pos[0], uv_pos[1], uv_size[0], uv_size[1]],
                            },
                            rot: 0.0,
                            origin: [0.5, 0.5],
                            scale: [1.0, 1.0],
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

#[derive(Debug, Clone)]
pub struct ActorImage {
    anim_state: FrameAnimState<Dir8, SpriteData>,
    state: DoubleSwap<ActorSnapshot>,
    /// Sec
    dt: f32,
    walk_secs: f32,
}

/// Interpolate two snapshots to draw actor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSnapshot {
    pos: Vec2i,
    dir: Dir8,
}

impl ActorImage {
    pub fn from_path(
        path: impl AsRef<Path>,
        anim_fps: f32,
        walk_secs: f32,
        pos: Vec2i,
        dir: Dir8,
    ) -> snow2d::gfx::tex::Result<Self> {
        let anim = self::gen_anim4(&Texture2dDrop::from_path(path)?.into_shared(), anim_fps);
        let mut anim_state = FrameAnimState::new(anim, dir);
        anim_state.set_pattern(dir, false);

        let data = ActorSnapshot { pos, dir };

        Ok(Self {
            anim_state,
            walk_secs,
            state: DoubleSwap::new(data, data),
            dt: Default::default(),
        })
    }

    pub fn force_set(&mut self, pos: Vec2i, dir: Dir8) {
        let next_snap = ActorSnapshot { dir, pos };
        self.state.a = next_snap;
        self.state.b = next_snap;
        self.anim_state.set_pattern(dir, true);
    }

    /// Call after updating actors
    pub fn update(&mut self, dt: Duration, pos: Vec2i, dir: Dir8) {
        if dir != self.state.a.dir {
            self.anim_state.set_pattern(dir, false);
        }

        // update interpolation value
        if pos != self.state.a.pos {
            self.dt = 0.0;
        }

        self.dt = f32::min(self.dt + dt.as_secs_f32(), self.walk_secs);

        let next_snap = ActorSnapshot { dir, pos };
        if next_snap != self.state.a {
            self.state.b = self.state.a;
            self.state.a = next_snap;
        }

        // update animation frame
        self.anim_state.tick(dt);
    }

    pub fn pos_screen(&self, tiled: &tiled::Map) -> Vec2f {
        let pos_prev = self.align(self.state.b.pos, tiled);
        let pos_curr = self.align(self.state.a.pos, tiled);

        let factor = self.dt / self.walk_secs;
        pos_prev * (1.0 - factor) + pos_curr * factor
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
        pos.y -= self.sprite().h() / 2.0;
        pos
    }

    /// Sprite for current frame
    pub fn sprite(&self) -> &SpriteData {
        self.anim_state.current_frame()
    }
}
