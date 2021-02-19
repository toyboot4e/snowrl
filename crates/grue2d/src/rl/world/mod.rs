//! The game world, internals and the GUI

pub mod actor;

use std::time::Duration;

use snow2d::Ice;

use rlbox::{
    render::camera::Camera2d,
    rl::{
        fov::{FovData, OpacityMap},
        fow::FowData,
        grid2d::*,
        rlmap::TiledRlMap,
    },
    utils::{arena::Arena, ez, Double},
};

use self::actor::*;

/// The rougelike game world
///
/// Turn-based game state should be outside of this struct.
#[derive(Debug)]
pub struct World {
    pub cam: Camera2d,
    pub map: TiledRlMap,
    pub shadow: Shadow,
    pub entities: Arena<Actor>,
}

/// Lifecycle
impl World {
    pub fn update(&mut self, ice: &mut Ice) {
        // FIXME: impl Into itermut
        for (_ix, e) in &mut self.entities {
            e.img.update(ice.dt, e.pos, e.dir);
        }
    }
}

/// API
impl World {
    pub fn player(&self) -> &Actor {
        let (ix, _) = self.entities.get_by_slot(0).unwrap();
        &self.entities[ix]
    }

    pub fn player_mut(&mut self) -> &mut Actor {
        let (ix, _) = self.entities.get_by_slot_mut(0).unwrap();
        &mut self.entities[ix]
    }

    pub fn is_blocked(&mut self, pos: Vec2i) -> bool {
        if self.map.rlmap.is_body_blocked(pos) {
            return true;
        }

        for (_ix, e) in &self.entities {
            if e.pos == pos {
                return true;
            }
        }

        false
    }
}

/// Shadow data for visualization
#[derive(Debug)]
pub struct Shadow {
    /// Field of view
    pub fov: Double<FovData>,
    /// Fog of war (shadow on map)
    pub fow: Double<FowData>,
    pub dt: ez::EasedDt,
    pub is_dirty: bool,
}

impl Shadow {
    pub fn new(radius: [u32; 2], map_size: [usize; 2], anim_secs: f32, ease: ez::Ease) -> Self {
        Self {
            fov: Double {
                a: FovData::new(radius[0], radius[1]),
                b: FovData::new(radius[0], radius[1]),
            },
            fow: Double {
                a: FowData::new(map_size),
                b: FowData::new(map_size),
            },
            dt: ez::EasedDt::new(anim_secs, ease),
            is_dirty: false,
        }
    }

    pub fn make_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn calculate(&mut self, origin: Vec2i, map: &impl OpacityMap) {
        // FoV is always cleared so we just swap them
        self.fov.swap();

        // FoW is continued from the previous state, so we'll copy it
        self.fow.b = self.fow.a.clone();

        self.dt.reset();

        rlbox::rl::fow::calculate_fov_fow(&mut self.fov.a, &mut self.fow.a, None, origin, map);
    }

    /// Call it every frame to animate FoV
    pub fn post_update(&mut self, dt: Duration, map: &impl OpacityMap, player: &Actor) {
        if self.is_dirty {
            self.calculate(player.pos, map);
            self.is_dirty = false;
        }

        self.dt.tick(dt);
    }
}
