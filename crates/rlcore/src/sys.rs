/*!
Roguelike core system

- GUI events (async events) are handled externally
- Internal events are injected via [`System`] trait
*/

use std::any::TypeId;

use snow2d::utils::arena::{Arena, Index};

/// Roguelike game system
pub trait System {
    /// Internal event type of the roguelike game system
    type Event;

    type EventTree;

    /// Actor type
    type Actor;

    fn next_actor(&mut self) -> Index<Self::Actor>;

    /// Decide actor action (UI | NonUI)
    fn take_turn(&mut self, ix: Index<Self::Actor>) -> EventData<Self::Event>;

    fn handle_event(&mut self, ev: Self::Event, tree: &mut Self::EventTree) -> HandleResult;
}

/// Return value of [`tick`](fn.tick.html)
#[derive(Debug, Clone, Default)]
pub struct TickResult<S: System> {
    pub gui: Option<UiEvent>,
    pub tree: S::EventTree,
}

/// Ticks the roguelike [`System`]
pub fn tick<S: System>(sys: &mut S) -> TickResult<S>
where
    S::EventTree: Default,
{
    let mut tree = S::EventTree::default();

    loop {
        let ix = sys.next_actor();
        let ev = sys.take_turn(ix);

        match ev {
            EventData::NonUI(ev) => {
                let res = sys.handle_event(ev, &mut tree);
                // TODO: handle result?
            }
            EventData::UI(gui) => {
                return TickResult {
                    gui: Some(gui),
                    tree,
                };
            }
        }
    }
}

/// Event handling result
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct HandleResult {
    pub is_turn_consuming: bool,
}

impl HandleResult {
    pub fn gui() -> Self {
        Self::default()
    }
}

/// `NonUI` | `UI`
#[derive(Debug)]
pub enum EventData<E> {
    NonUI(E),
    UI(UiEvent),
}

/// UI events are handled externally (by UI)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiEvent {
    // static, unique ID
    id: TypeId,
}

impl UiEvent {
    pub fn new(id: TypeId) -> Self {
        Self { id }
    }
}

type Slot = u32;

/// Utility for implementing [`System::next_actor`]
#[derive(Debug, Clone, Default)]
pub struct ActorSlot {
    slot: Slot,
}

impl ActorSlot {
    pub fn next<T>(&mut self, arena: &mut Arena<T>) -> Option<Index<T>> {
        let slot = self.slot;
        self.slot += 1;
        arena.len() as Slot;
        arena.get_by_slot(self.slot).map(|(ix, _data)| ix)
    }
}
