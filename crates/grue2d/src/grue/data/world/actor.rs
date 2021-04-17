/*!
Game entity with GUI
*/

use serde::{Deserialize, Serialize};

use snow2d::{
    ui::Layer,
    utils::{
        arena::Index,
        tyobj::{SerdeRepr, TypeObject, TypeObjectId},
    },
};

use rlbox::{
    rl::grid2d::*,
    view::actor::{ActorImage, ActorImageType, ActorNodes},
};

use crate::grue::data::world::World;

/// Internal and view states of an actor
#[derive(Debug, Clone)]
pub struct Actor {
    pub pos: Vec2i,
    pub dir: Dir8,
    pub stats: ActorStats,
    pub view: ActorImage,
    pub nodes: ActorNodes,
    pub relation: Relation,
    pub interact: Option<Interactable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorStats {
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
}

/// Relation with player: `Hostile` | `Friendly`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Relation {
    Hostile,
    Friendly,
}

/// Type object for [`Actor`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorType {
    pub img: SerdeRepr<ActorImageType>,
    pub stats: ActorStats,
}

impl TypeObject for ActorType {}

/// [`Actor`] component
#[derive(Debug, Clone)]
pub struct Interactable {
    //
}

/// TODO: node-based script implementation and DSL
#[derive(Debug)]
pub struct Script {
    //
}

impl TypeObject for Script {}

// --------------------------------------------------------------------------------
// Data-driven content

/// Create [`Actor`] from RON files
#[derive(Debug, Clone)]
pub struct ActorSpawn {
    pub type_id: TypeObjectId<ActorType>,
    pub pos: Vec2i,
    pub dir: Dir8,
    pub relation: Relation,
}

impl ActorSpawn {
    pub fn new(type_: impl Into<TypeObjectId<ActorType>>) -> Self {
        Self {
            type_id: type_.into(),
            pos: Vec2i::default(),
            dir: Dir8::S,
            relation: Relation::Friendly,
        }
    }

    pub fn pos(&mut self, pos: impl Into<Vec2i>) -> &mut Self {
        self.pos = pos.into();
        self
    }

    pub fn dir(&mut self, dir: Dir8) -> &mut Self {
        self.dir = dir;
        self
    }

    pub fn relation(&mut self, rel: Relation) -> &mut Self {
        self.relation = rel;
        self
    }

    pub fn hostile(&mut self) -> &mut Self {
        self.relation = Relation::Hostile;
        self
    }

    pub fn friendly(&mut self) -> &mut Self {
        self.relation = Relation::Friendly;
        self
    }

    pub fn spawn(&self, world: &mut World, layer: &mut Layer) -> anyhow::Result<Index<Actor>> {
        let type_ = ActorType::from_type_key(&self.type_id)?;
        let img: ActorImage = type_
            .img
            .map(|desc| ActorImage::from_desc_default(desc))
            .unwrap();

        let nodes = ActorNodes::new(layer, img.sprite());

        let mut actor = Actor {
            pos: self.pos,
            dir: self.dir,
            view: img,
            stats: type_.stats.clone(),
            nodes,
            relation: self.relation,
            interact: None,
        };

        actor.view.warp(self.pos, self.dir);

        Ok(world.entities.insert(actor))
    }
}
