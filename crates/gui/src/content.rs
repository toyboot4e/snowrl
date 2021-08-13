/*!
Game content
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
    entity::{ActorStats, EntityModel, Relation},
    EventData, Model,
};
use view::actor::{ActorImage, ActorImageType, ActorNodes, ActorView};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerAi;

impl model::entity::Ai for PlayerAi {
    fn take_turn(&self, _entity: Index<EntityModel>, _model: &mut Model) -> Option<EventData> {
        Some(model::evs::PlayerCommand.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoneAi;

impl model::entity::Ai for NoneAi {
    fn take_turn(&self, _entity: Index<EntityModel>, _model: &mut Model) -> Option<EventData> {
        None
    }
}
