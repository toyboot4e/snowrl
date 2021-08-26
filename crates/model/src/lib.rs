/*!
SnowRL internal game states
*/

pub extern crate core;

pub mod chg;
pub mod entity;
pub mod evs;
pub mod map;

use std::{collections::HashMap, fmt};

use snow2d::utils::arena::{Arena, Index};

use rlcore::{
    ev::{
        hub::{DynEvent, EventHub},
        tree::EventTree,
        HubSystem, SystemArgs,
    },
    sys::{ActorSlot, HandleResult},
};

use crate::{entity::*, map::MapModel};

/// Upcasted event data
pub type EventData = rlcore::sys::EventData<DynEvent, chg::Change>;

/// Event hub builder, where you register your events and event handlers
pub type EventHubBuilder = rlcore::ev::hub::EventHubBuilder<GameSystem>;

/// Roguelike game system than can be [`tick`](rlcore::tick)ed
#[derive(Debug)]
pub struct GameSystem {
    /// Turn-based game state
    slot: ActorSlot,
    /// Internal game state
    model: Model,
    /// Event handling dispatcher
    pub hub: EventHub<Self>,
    /// Behavior logic dispatcher
    pub ais: AiHub,
}

impl GameSystem {
    pub fn new(model: Model) -> Self {
        Self {
            slot: ActorSlot::default(),
            model,
            hub: Default::default(),
            ais: Default::default(),
        }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    /// WARNING: Changes without autmatic visualization can cause diff between GUI and internal!
    pub fn make_immediate_change(&mut self, vm: &mut Model, chg: &chg::Change) {
        use rlcore::ev::Model;
        self.model.apply_change(chg);
        vm.apply_change(chg);
    }
}

impl rlcore::sys::System for GameSystem {
    type Event = DynEvent;
    type EventTree = EventTree;
    type Entity = EntityModel;
    type Change = chg::Change;

    fn _next_actor(&mut self) -> Index<Self::Entity> {
        self.slot.next(&mut self.model.entities).unwrap()
    }

    fn _take_turn(&mut self, ix: Index<Self::Entity>) -> Option<EventData> {
        let model = &self.model.entities[ix];
        let ai = model.ai.clone();
        self.ais.take_turn(&ai, ix, &mut self.model)
    }

    fn _handle_event(&mut self, ev: Self::Event, _tree: &mut Self::EventTree) -> HandleResult {
        // TODO: Don't swap.
        let mut model = Model::default();
        std::mem::swap(&mut model, &mut self.model);
        let mut args = SystemArgs::new(model);

        let res = self.hub.handle(&ev, &mut args);
        let (model, _builder) = args.retrieve();
        self.model = model;

        res
    }

    /// Applies the mutation to the game state
    fn _apply_change(&mut self, chg: &Self::Change) {
        use rlcore::ev::Model;
        self.model.apply_change(chg);
    }
}

impl HubSystem for GameSystem {
    type Args = SystemArgs<Model>;
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

impl rlcore::ev::Model for Model {
    type Change = chg::Change;
    fn apply_change(&mut self, chg: &Self::Change) {
        chg.apply(self);
    }
}
