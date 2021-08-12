/*!
SnowRL internal game states
*/

pub extern crate core;

pub mod entity;
pub mod evs;
pub mod map;

use rlcore::ev::{tree::EventTree, Event};

use snow2d::utils::arena::{Arena, Index};

use rlcore::{
    ev::{
        hub::{EventHub, HubSystem},
        tree::EventSystem,
    },
    sys::{ActorSlot, HandleResult},
};

use crate::{entity::*, map::MapModel};

pub type EventData = rlcore::sys::EventData<RlEvent>;

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

pub type EventHubBuilder = rlcore::ev::hub::EventHubBuilder<GameSystem>;

/// Internal game state of SnowRL
#[derive(Debug, Clone, Default)]
pub struct Model {
    pub entities: Arena<EntityModel>,
    pub map: MapModel,
}

impl rlcore::sys::System for GameSystem {
    type Event = RlEvent;
    type EventTree = EventTree;
    type Entity = EntityModel;

    fn next_actor(&mut self) -> Index<Self::Entity> {
        self.slot.next(&mut self.model.entities).unwrap()
    }

    fn take_turn(&mut self, ix: Index<Self::Entity>) -> Option<EventData> {
        let model = &self.model.entities[ix];
        let ai = model.ai.clone();
        ai.take_turn(ix, &mut self.model)
    }

    fn handle_event(&mut self, ev: Self::Event, tree: &mut Self::EventTree) -> HandleResult {
        match ev {
            RlEvent::Inline(InlineEvent) => {
                todo!()
            }
            RlEvent::Dyn(ev) => {
                let _access = EventSystem::new(&mut self.model);
                self.hub.handle_any(ev, &mut self.model)
            }
        };

        todo!()
    }
}

impl HubSystem for GameSystem {
    type Context = Model;
}
