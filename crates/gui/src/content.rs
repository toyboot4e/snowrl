/*!
Game content in GUI world
*/

use snow2d::utils::arena::Index;

use rlcore::{
    ev::{hub::EventHubBuilder, Event},
    grid2d::*,
    sys::{UiEvent, UiEventTag},
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
        let es = &model.entities;
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

/// Tag for AI and GUI event
// TODO: add deriva event for generating the tag[s]
pub struct PlayerAi;

impl UiEvent for PlayerAi {}

impl PlayerAi {
    pub const AI: AiTag = AiTag::new("player");
    /// TODO: pub const and easy match
    pub const GUI: &'static str = "player-ai";

    /// Always yield [`PlayerAi`] UI event
    pub fn logic(_entity: Index<EntityModel>, _model: &mut Model) -> Option<EventData> {
        // FIXME: Don't alloc String
        Some(EventData::UI(UiEventTag::new(Self::GUI.to_string())))
    }
}
