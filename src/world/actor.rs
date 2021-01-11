/*!

Game entity with GUI

Ideally, internals should be separated from graphics, but coupling them would be good for
prototpyes.

*/

use rlbox::{render::actor::ActorImage, rl::grid2d::*};

#[derive(Debug, Clone)]
pub struct Actor {
    pub pos: Vec2i,
    pub dir: Dir8,
    pub img: ActorImage,
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
