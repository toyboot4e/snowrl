//! Actor

use std::{path::Path, time::Duration};

use snow2d::gfx::{
    batcher::draw::*,
    tex::{SpriteData, Texture2dDrop},
};

use rlbox::{
    anim::FrameAnimState,
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
    prev_pos: Vec2i,
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

        Ok(Self {
            anim_state: FrameAnimState::new(anim, dir),
            dir,
            prev_pos: pos,
        })
    }

    pub fn update(&mut self, dt: Duration) {
        self.anim_state.tick(dt);
    }

    pub fn render(&mut self, draw: &mut impl DrawApi, tiled: &tiled::Map) {
        // TODO: lerp
    }

    pub fn before_walk(&mut self, prev_pos: Vec2i, new_dir: Dir8) {
        self.prev_pos = prev_pos;
        self.dir = new_dir;
        self.anim_state.set_pattern(new_dir, false);
    }

    /// Current sprite frame
    pub fn sprite(&self) -> &SpriteData {
        self.anim_state.current_frame()
    }
}
