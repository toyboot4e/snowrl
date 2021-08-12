/*!
Actor type and components
*/

use serde::{Deserialize, Serialize};

use snow2d::utils::{arena::Index, Inspect};

use rlcore::grid2d::*;

use crate::EventData;

/// Fixed set of components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityModel {
    pub pos: Vec2i,
    pub dir: Dir8,
    pub stats: ActorStats,
    pub relation: Relation,
    // interact: Interact,
}

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
