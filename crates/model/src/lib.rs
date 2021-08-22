/*!
SnowRL internal game states
*/

pub extern crate core;

pub mod entity;
pub mod evs;
pub mod map;

use std::{collections::HashMap, fmt};

use rlcore::ev::tree::EventTree;

use snow2d::utils::arena::{Arena, Index};

use rlcore::{
    ev::{
        hub::{DynEvent, EventHub, HubSystem},
        tree::EventSystem,
    },
    sys::{ActorSlot, HandleResult},
};

use crate::{entity::*, map::MapModel};

/// Upcasted event data
pub type EventData = rlcore::sys::EventData<DynEvent>;

/// Event hub builder, where you register your events and event handlers
pub type EventHubBuilder = rlcore::ev::hub::EventHubBuilder<GameSystem>;

/// Roguelike game system than can be [`tick`](rlcore::tick)ed
#[derive(Debug, Default)]
pub struct GameSystem {
    /// Turn-based game state
    slot: ActorSlot,
    /// Internal game state
    pub model: Model,
    /// Event handlers
    pub hub: EventHub<Self>,
    /// Behavior logics
    pub ais: AiHub,
}

impl rlcore::sys::System for GameSystem {
    type Event = DynEvent;
    type EventTree = EventTree;
    type Entity = EntityModel;

    fn next_actor(&mut self) -> Index<Self::Entity> {
        self.slot.next(&mut self.model.entities).unwrap()
    }

    fn take_turn(&mut self, ix: Index<Self::Entity>) -> Option<EventData> {
        let model = &self.model.entities[ix];
        let ai = model.ai.clone();
        self.ais.take_turn(&ai, ix, &mut self.model)
    }

    fn handle_event(&mut self, ev: Self::Event, _tree: &mut Self::EventTree) -> HandleResult {
        let _access = EventSystem::new(&mut self.model);
        self.hub.handle(&ev, &mut self.model)
    }
}

impl HubSystem for GameSystem {
    type Context = Model;
}

/// Upcasted AI logic
pub type AiLogic = Box<dyn FnMut(Index<EntityModel>, &mut Model) -> Option<EventData>>;

/// Dispatches AI logic to [`AiTag`]
#[derive(Default)]
pub struct AiHub {
    logics: HashMap<AiTag, AiLogic>,
}

impl fmt::Debug for AiHub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.logics.keys().fmt(f)
    }
}

impl AiHub {
    /// Register new AI
    pub fn add(&mut self, tag: AiTag, logic: AiLogic) -> &mut Self {
        assert!(
            self.logics.insert(tag, logic).is_none(),
            "Duplicate AI logics"
        );
        self
    }

    /// Dispatches and runs AI logic
    pub fn take_turn(
        &mut self,
        ai: &AiTag,
        index: Index<EntityModel>,
        model: &mut Model,
    ) -> Option<EventData> {
        let logic = self
            .logics
            .get_mut(ai)
            .unwrap_or_else(|| panic!("Unable to find logic for AI tag {:?}", ai));
        (logic)(index, model)
    }
}

/// Internal game state of SnowRL
#[derive(Debug, Clone, Default)]
pub struct Model {
    pub entities: Arena<EntityModel>,
    pub map: MapModel,
}
