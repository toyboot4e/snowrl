/*!
Roguelike core system

- GUI events (async events) are handled externally
- Internal events are injected via [`System`] trait
*/

use std::any::{self, TypeId};

use snow2d::utils::arena::{Arena, Index, Slot};

/// Roguelike game system
pub trait System {
    /// Internal event type of the roguelike game system
    type Event;

    type Model: crate::ev::Model;

    type EventTree;

    /// Actor type
    type Entity;

    /// (tick)
    fn _next_actor(&mut self) -> Index<Self::Entity>;

    /// (tick) Decide actor action (UI | NonUI)
    fn _take_turn(
        &mut self,
        ix: Index<Self::Entity>,
    ) -> Option<EventData<Self::Event, <Self::Model as crate::ev::Model>::Change>>;

    /// (tick)
    fn _handle_event(&mut self, ev: Self::Event, tree: &mut Self::EventTree);

    /// (tick) Applies the mutation to the game state
    fn _apply_change(&mut self, chg: &<Self::Model as crate::ev::Model>::Change);
}

/// Return value of [`tick`](fn.tick.html)
#[derive(Debug, Clone, Default)]
pub struct TickResult<S: System> {
    pub gui: Option<UiEventTag>,
    pub tree: S::EventTree,
}

/// Ticks the roguelike [`System`]
pub fn tick<S: System>(sys: &mut S) -> TickResult<S>
where
    S::EventTree: Default,
{
    let mut tree = S::EventTree::default();

    loop {
        let ix = sys._next_actor();
        let ev = match sys._take_turn(ix) {
            Some(ev) => ev,
            // next actor
            None => continue,
        };

        match ev {
            EventData::Action(ev) => {
                let _res = sys._handle_event(ev, &mut tree);
                // TODO: handle result?
            }
            EventData::Change(chg) => {
                sys._apply_change(&chg);
            }
            EventData::UI(gui) => {
                break TickResult {
                    gui: Some(gui),
                    tree,
                };
            }
        }
    }
}

/// `Action` | `Change` | `UI`
#[derive(Debug)]
pub enum EventData<E, C> {
    /// Action event is something that is dispatched handler
    Action(E),
    /// Change is a mutation to the game world
    Change(C),
    /// UI events are tags and handled externally by UI
    UI(UiEventTag),
}

/// Marker for UI event types
pub trait UiEvent {}

impl<E, C, T: UiEvent + 'static> From<T> for EventData<E, C> {
    fn from(_ev: T) -> Self {
        Self::UI(UiEventTag {
            raw: any::type_name::<T>().to_string(),
        })
    }
}

/// UI events are just tags and handled externally (by UI)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiEventTag {
    raw: String,
}

impl UiEventTag {
    /// FIXME: Use derive to create hidden, unique ID
    pub fn new(s: String) -> Self {
        Self { raw: s }
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

/// Utility for implementing [`System::_next_actor`]
#[derive(Debug, Clone, Default)]
pub struct ActorSlot {
    slot: Slot,
}

impl ActorSlot {
    pub fn next<T>(&mut self, arena: &mut Arena<T>) -> Option<Index<T>> {
        let index = {
            self.slot = Slot::from_raw(self.slot.raw() % arena.capacity() as u32);

            // TODO: stop on infinite loop
            // let origin = self.slot;
            let index = loop {
                let slot = self.slot;
                self.slot = Slot::from_raw(self.slot.raw() + 1);
                if let Some(index) = arena.upgrade(slot) {
                    break index;
                }
            };

            index
        };

        Some(index)
    }
}
