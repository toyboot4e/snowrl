use snow2d::utils::arena::Index;

use rlcore::sys::UiEvent;

use model::{
    entity::{AiTag, EntityModel},
    EventData, Model,
};

pub struct PlayerAi;

impl UiEvent for PlayerAi {}

impl PlayerAi {
    pub const TAG: AiTag = AiTag::new("player");

    /// Always yield [`PlayerAi`] UI event
    pub fn logic(_entity: Index<EntityModel>, _model: &mut Model) -> Option<EventData> {
        Some(PlayerAi.into())
    }
}
