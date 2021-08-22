/*!
Game content in GUI world
*/

use snow2d::utils::arena::Index;

use rlcore::{
    ev::{hub::EventHubBuilder, Event},
    grid2d::*,
    sys::UiEvent,
};

use model::{
    entity::{AiTag, EntityModel},
    EventData, GameSystem, Model,
};

// --------------------------------------------------------------------------------
// Event

/// Registers model events and default event handlers to [`EventHubBuilder`]
pub fn builder_plugin(builder: &mut EventHubBuilder<GameSystem>) {
    builder.ev_with(Box::new(|ev: &Interact, model| {
        let es = &mut model.entities;
        let dir = ev.dir.unwrap_or_else(|| es[ev.entity].dir);
        let pos = es[ev.entity].pos + Vec2i::from(dir);
        let _target = match es.items().find(|e| e.pos == pos) {
            Some(e) => e,
            None => return None,
        };
        // TODO: interaction handling
        None
    }));
}

#[derive(Debug, Clone)]
pub struct Interact {
    pub entity: Index<EntityModel>,
    pub dir: Option<Dir8>,
}

// TODO: add derive macro
impl Event for Interact {}

// --------------------------------------------------------------------------------
// AI

pub struct PlayerAi;

impl UiEvent for PlayerAi {}

impl PlayerAi {
    pub const TAG: AiTag = AiTag::new("player");

    /// Always yield [`PlayerAi`] UI event
    pub fn logic(_entity: Index<EntityModel>, _model: &mut Model) -> Option<EventData> {
        Some(PlayerAi.into())
    }
}
