/*!
Game entity with GUI
*/

use serde::{Deserialize, Serialize};

use snow2d::{
    ui,
    utils::{
        arena::Index,
        tyobj::{SerdeRepr, TypeObject, TypeObjectId},
    },
};

use rlbox::{
    rl::grid2d::*,
    view::actor::{ActorImage, ActorImageDesc, ActorNodes},
};

use crate::data::{
    resources::{Ui, UiLayer},
    world::World,
};

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

/// Runtime represntation of actor
#[derive(Debug, Clone)]
pub struct Actor {
    pub pos: Vec2i,
    pub dir: Dir8,
    pub img: ActorImage,
    pub stats: ActorStats,
    pub nodes: ActorNodes,
}

/// Create [`Actor`] from RON files
#[derive(Debug, Clone)]
pub struct ActorSpawn {
    pub type_: TypeObjectId<ActorType>,
    pub pos: Vec2i,
    pub dir: Dir8,
}

impl ActorSpawn {
    pub fn new(type_: impl Into<TypeObjectId<ActorType>>) -> Self {
        Self {
            type_: type_.into(),
            pos: Vec2i::default(),
            dir: Dir8::S,
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

    pub fn spawn(&self, world: &mut World, ui: &mut Ui) -> anyhow::Result<Index<Actor>> {
        let type_ = ActorType::from_type_key(&"ika-chan".into())?;
        let img: ActorImage = type_
            .img
            .map(|desc| ActorImage::from_desc_default(desc))
            .unwrap();

        let layer = ui.get_mut(UiLayer::Actors);
        let nodes = ActorNodes {
            img: layer.nodes.add(ui::node::Text {
                txt: "a".to_string(),
            }),
            hp: layer.nodes.add(img.sprite()),
        };

        let mut actor = Actor {
            pos: self.pos,
            dir: self.dir,
            img,
            stats: type_.stats.clone(),
            nodes,
        };
        actor.img.warp(self.pos, self.dir);

        Ok(world.entities.insert(actor))
    }
}
