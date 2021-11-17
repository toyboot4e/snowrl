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

use model::entity::{ActorStats, AiTag, EntityModel, Relation};

use view::actor::{ActorImage, ActorImageType, ActorNodes, ActorView};

use crate::{res::UiLayer, Gui};

/// Type object for model/view of actor
#[derive(Debug, Clone, Serialize, Deserialize, TypeObject)]
pub struct ActorType {
    pub img: SerdeRepr<ActorImageType>,
    pub ai: AiTag,
    pub stats: ActorStats,
}

/// Create actor easily
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

        let mdl = {
            let actor_mdl = EntityModel {
                pos: self.pos,
                dir: self.dir,
                stats: type_.stats.clone(),
                relation: self.relation,
                ai: type_.ai.clone(),
            };
            gui.vm.entities.insert(actor_mdl)
        };

        let mut view = {
            let img: ActorImage = type_
                .img
                .map(|desc| ActorImage::from_desc_default(desc))
                .unwrap();

            let nodes = ActorNodes::new(ui, UiLayer::Actors.to_layer(), img.sprite());

            ActorView { mdl, img, nodes }
        };

        view.img.warp(self.pos, self.dir);

        Ok(gui.actors.insert(view))
    }
}
