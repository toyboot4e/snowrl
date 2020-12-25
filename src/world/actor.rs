//! Actor

use std::{path::Path, time::Duration};

use snow2d::gfx::{
    batcher::draw::*,
    geom2d::*,
    tex::{SpriteData, Texture2dDrop},
};

use rlbox::{
    render::anim::FrameAnimState,
    rl::{fov::FovData, grid2d::*},
};

#[derive(Debug, Clone)]
pub struct Player {
    pub pos: Vec2i,
    pub dir: Dir8,
    pub fov: FovData,
    pub img: ActorImage,
}

#[derive(Debug, Clone)]
pub struct ActorImage {
    anim_state: FrameAnimState<Dir8, SpriteData>,
    state: Double<ActorSnapshot>,
    /// Sec
    dt: f32,
}

#[derive(Debug, Clone)]
struct Double<T> {
    pub a: T,
    pub b: T,
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
        pos: Vec2i,
        dir: Dir8,
    ) -> snow2d::gfx::tex::Result<Self> {
        let tex = Texture2dDrop::from_path(path)?;
        let tex = tex.into_shared();
        let anim = rlbox::render::actor::gen_anim4(&tex, crate::consts::ACTOR_FPS);

        let data = ActorSnapshot { pos, dir };

        Ok(Self {
            anim_state: FrameAnimState::new(anim, dir),
            state: Double { a: data, b: data },
            dt: Default::default(),
        })
    }

    /// Call after updating actors
    pub fn update(&mut self, dt: Duration, pos: Vec2i, dir: Dir8) {
        if dir != self.state.a.dir {
            self.anim_state.set_pattern(dir, true);
        }

        if pos != self.state.a.pos {
            self.dt = 0.0;
        } else {
            self.dt = f32::min(self.dt + dt.as_secs_f32(), crate::consts::WALK_TIME);
        }

        let next_snap = ActorSnapshot { dir, pos };
        if next_snap != self.state.a {
            self.state.b = self.state.a;
            self.state.a = next_snap;
        }

        self.anim_state.tick(dt);
    }

    pub fn render(&self, draw: &mut impl DrawApi, tiled: &tiled::Map) {
        let pos_prev = self.align(self.state.b.pos, tiled);
        let pos_curr = self.align(self.state.a.pos, tiled);

        let factor = self.dt / crate::consts::WALK_TIME;
        let pos = pos_prev + (pos_curr - pos_prev) * factor;

        draw.sprite(self.sprite()).dst_pos_px(pos);
    }

    /// Align the bottom-center of an actor to the bottom-center of a cell
    fn align(&self, pos: Vec2i, tiled: &tiled::Map) -> Vec2f {
        let mut pos = rlbox::render::tiled::t2w_center(pos, &tiled);
        pos.y += tiled.tile_height as f32 / 2.0;
        pos.y -= self.sprite().h() / 2.0;
        pos
    }

    /// Sprite for current frame
    pub fn sprite(&self) -> &SpriteData {
        self.anim_state.current_frame()
    }
}
