/*!
Actor type and components
*/

use std::fmt;

use derivative::Derivative;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

use snow2d::utils::{arena::Index, Inspect};

use rlcore::grid2d::*;

use crate::{EventData, Model};

/// Fixed set of components
#[derive(Debug, Clone)]
pub struct EntityModel {
    pub pos: Vec2i,
    pub dir: Dir8,
    pub stats: ActorStats,
    pub relation: Relation,
    // interact: Interact,
    pub ai: Box<dyn Ai>,
}

pub trait Ai: fmt::Debug + DynClone {
    fn take_turn(&self, entity: Index<EntityModel>, model: &mut Model) -> Option<EventData>;
}

dyn_clone::clone_trait_object!(Ai);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Inspect)]
pub struct ActorStats {
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
}

/// Relation with player: `Hostile` | `Friendly`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Inspect)]
pub enum Relation {
    Hostile,
    Friendly,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub use_: ItemUse,
}

#[derive(Debug, Clone)]
pub struct ItemUse {}
