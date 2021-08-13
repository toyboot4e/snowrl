/*!
Spawn game content
*/

use serde::{Deserialize, Serialize};

use snow2d::{
    ui::Ui,
    utils::{
        arena::Index,
        tyobj::{SerdeRepr, TypeObject, TypeObjectId},
        Inspect,
    },
};

use rlcore::grid2d::*;

use model::{
    entity::{ActorStats, Ai, EntityModel, Relation},
    Model,
};

use view::actor::{ActorImage, ActorImageType, ActorNodes, ActorView};

use crate::{content::PlayerAi, res::UiLayer, Gui};

/// Type object for model/view of actor
#[derive(Debug, Clone, Serialize, Deserialize, TypeObject)]
pub struct ActorType {
    pub img: SerdeRepr<ActorImageType>,
    pub ai: AiType,
    pub stats: ActorStats,
}

// cheap, so not stored in type object storage
// TODO: add derive macro to define type and verify there's no dups with static storage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AiType(String);

impl AiType {
    pub fn to_ai(&self) -> Box<dyn Ai> {
        match self.0.as_str() {
            "player" => Box::new(PlayerAi),
            _ => panic!("invalid "),
        }
    }
}

/// Create [`Actor`] easily
#[derive(Debug, Clone, Serialize, Deserialize, Inspect)]
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

    /// TODO: Trun spawn into event
    pub fn spawn_to_gui(&self, gui: &mut Gui, ui: &mut Ui) -> anyhow::Result<Index<ActorView>> {
        let type_ = ActorType::from_type_key(&self.type_id)?;

        let model = {
            let actor_model = EntityModel {
                pos: self.pos,
                dir: self.dir,
                stats: type_.stats.clone(),
                relation: self.relation,
                ai: Box::new(PlayerAi),
            };
            gui.vm.entities.insert(actor_model)
        };

        let mut view = {
            let img: ActorImage = type_
                .img
                .map(|desc| ActorImage::from_desc_default(desc))
                .unwrap();

            let nodes = ActorNodes::new(ui, UiLayer::Actors.to_layer(), img.sprite());

            ActorView { model, img, nodes }
        };

        view.img.warp(self.pos, self.dir);

        Ok(gui.entities.insert(view))
    }
}
