/*!

Game entity with GUI

Ideally, internals should be separated from graphics, but coupling them would be good for
prototpyes.

*/

use serde::{Deserialize, Serialize};

use snow2d::utils::type_object::TypeObject;

use rlbox::{
    rl::grid2d::*,
    view::actor::{ActorImage, ActorImageSerde},
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

/// FIXME: allow reference
pub struct TypeObjectId(String);

impl TypeObjectId {
    pub fn from_raw(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn raw(&self) -> &str {
        &self.0
    }
}

/// Type object for [`Actor`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorType {
    pub stats: ActorStats,
    pub img: ActorImageSerde,
}

impl TypeObject for ActorType {}

impl ActorType {
    pub fn to_actor(&self) -> Actor {
        Actor {
            pos: Default::default(),
            dir: Dir8::S,
            img: ActorImage::from_serde_repr_default(&self.img),
            stats: self.stats.clone(),
        }
    }
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
