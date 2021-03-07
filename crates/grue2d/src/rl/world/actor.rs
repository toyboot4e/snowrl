/*!

Game entity with GUI

Ideally, internals should be separated from graphics, but coupling them would be good for
prototpyes.

*/

use serde::{Deserialize, Serialize};

use snow2d::utils::tyobj::{SerdeRepr, TypeObject};

use rlbox::{
    rl::grid2d::*,
    view::actor::{ActorImage, ActorImageDesc},
};

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

/// Type object for [`Actor`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorType {
    pub img: SerdeRepr<ActorImageDesc>,
    pub stats: ActorStats,
}

impl TypeObject for ActorType {}

impl ActorType {
    pub fn to_actor(&self) -> Actor {
        Actor {
            pos: Default::default(),
            dir: Dir8::S,
            img: self
                .img
                .map(|desc| ActorImage::from_desc_default(desc))
                .unwrap(),
            stats: self.stats.clone(),
        }
    }
}

// pub trait Behavior {
//     fn gen_ev(&mut self, bcx: &mut BehaviorContext) -> Box<dyn Event>;
// }
