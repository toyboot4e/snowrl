/*!
Entity model and components
*/

use std::{borrow::Borrow, borrow::Cow};

use serde::{Deserialize, Serialize};

use snow2d::utils::Inspect;

use rlcore::grid2d::*;

/// Fixed set of components
#[derive(Debug, Clone)]
pub struct EntityModel {
    pub pos: Vec2i,
    pub dir: Dir8,
    pub stats: ActorStats,
    pub relation: Relation,
    // interact: Interact,
    pub ai: AiTag,
}

/// Tag of AI which is resolved to a specific logic by system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AiTag(Cow<'static, str>);

impl AiTag {
    pub const fn new(tag: &'static str) -> Self {
        Self(Cow::Borrowed(tag))
    }

    pub fn tag(&self) -> &str {
        self.0.borrow()
    }
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
