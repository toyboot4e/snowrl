/*!

Game entity with GUI

Ideally, internals should be separated from graphics, but coupling them would be good for
prototpyes.

*/

use rlbox::{rl::grid2d::*, view::actor::ActorImage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub pos: Vec2i,
    pub dir: Dir8,
    pub img: ActorImage,
    pub stats: ActorStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorStats {
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
}

// pub struct ActorList {
//     actors: Vec<Actor>,
// }

// pub struct ActorId {
//     /// Index
//     ix: usize,
//     /// Generation
//     gen: usize,
// }

// pub trait Behavior {
//     fn gen_ev(&mut self, bcx: &mut BehaviorContext) -> Box<dyn Event>;
// }
