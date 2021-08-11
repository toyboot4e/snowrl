/*!
SnowRL internal game states
*/

pub extern crate core;

pub mod entity;
pub mod evs;

use core::ev::Event;

use snow2d::utils::arena::{Arena, Index};

use core::{
    ev::{
        hub::{EventHub, HubSystem},
        tree::EventSystem,
    },
    map::MapModel,
    sys::{ActorSlot, EventData, HandleResult},
};

use crate::entity::*;

/// [`InlineEvent`] | [`DynEvent`]
#[derive(Debug, Clone)]
pub enum RlEvent {
    Inline(InlineEvent),
    Dyn(DynEvent),
}

/// Builtin on-stack event
#[derive(Debug, Clone)]
pub enum InlineEvent {
    Spawn { actor: EntityModel },
}

/// Heap-allocated event
pub type DynEvent = Box<dyn Event>;

/// Roguelike game system
#[derive(Debug, Default)]
pub struct GameSystem {
    slot: ActorSlot,
    pub model: Model,
    pub hub: EventHub<Self>,
}

pub type EventHubBuilder = core::ev::hub::EventHubBuilder<GameSystem>;

/// Internal game state of SnowRL
#[derive(Debug, Clone, Default)]
pub struct Model {
    pub entities: Arena<EntityModel>,
    pub map: MapModel,
}

impl core::sys::System for GameSystem {
    type Event = RlEvent;
    type Actor = EntityModel;

    fn next_actor(&mut self) -> Index<Self::Actor> {
        self.slot.next(&mut self.model.entities).unwrap()
    }

    fn take_turn(&mut self, ix: Index<Self::Actor>) -> EventData<Self::Event> {
        todo!()
    }

    fn handle_event(&mut self, ev: Self::Event) -> HandleResult {
        match ev {
            RlEvent::Inline(InlineEvent) => {
                todo!()
            }
            RlEvent::Dyn(ev) => {
                let access = EventSystem::new(&mut self.model);
                self.hub.handle_any(ev, &mut self.model)
            }
        };

        todo!()
    }
}

impl HubSystem for GameSystem {
    type Context = Model;
}
