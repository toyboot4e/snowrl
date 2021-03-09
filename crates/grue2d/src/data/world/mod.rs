/*!
The game world, internals and the GUI
*/

pub mod actor;

use snow2d::{utils::arena::Arena, Ice};

use rlbox::{
    rl::grid2d::*,
    view::{
        camera::{Camera2d, FollowCamera2d},
        map::TiledRlMap,
        shadow::Shadow,
    },
};

use self::actor::*;

pub type Entities = Arena<Actor>;

/// The rougelike game world
///
/// Turn-based game state should be outside of this struct.
#[derive(Debug)]
pub struct World {
    pub cam: Camera2d,
    pub cam_follow: FollowCamera2d,
    pub map: TiledRlMap,
    pub shadow: Shadow,
    pub entities: Entities,
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
