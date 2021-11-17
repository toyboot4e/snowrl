/*!
Game content in GUI world
*/

use snow2d::utils::arena::Index;

use rlcore::{
    ev::Event,
    grid2d::*,
    sys::{UiEvent, UiEventTag},
};

use model::{
    entity::{AiTag, EntityModel},
    EventData, EventHubBuilder, HandleResult, Model,
};

// --------------------------------------------------------------------------------
// Event

/// Registers model events and default event handlers to [`EventHubBuilder`]
pub fn builder_plugin(builder: &mut EventHubBuilder) {
    builder.ev_with(Box::new(|ev: &Interact, args| {
        let es = &args.ents;
        let dir = ev.dir.unwrap_or_else(|| es[ev.ent].dir);
        let pos = es[ev.ent].pos + Vec2i::from(dir);

        let _target = match es.items().find(|e| e.pos == pos) {
            Some(e) => e,
            None => return HandleResult::Handled,
        };

        // TODO: interaction handling
        HandleResult::Handled
    }));
}

#[derive(Debug, Clone)]
pub struct Interact {
    pub ent: Index<EntityModel>,
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
    pub fn logic(_entity: Index<EntityModel>, _mdl: &mut Model) -> Option<EventData> {
        // FIXME: Don't alloc String
        Some(EventData::UI(UiEventTag::new(Self::GUI.to_string())))
    }
}
