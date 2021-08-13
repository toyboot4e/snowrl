/*!
Roguelike core system

- GUI events (async events) are handled externally
- Internal events are injected via [`System`] trait
*/

use std::any;

use snow2d::utils::arena::{Arena, Index};

/// Roguelike game system
pub trait System {
    /// Internal event type of the roguelike game system
    type Event;

    type EventTree;

    /// Actor type
    type Entity;

    fn next_actor(&mut self) -> Index<Self::Entity>;

    /// Decide actor action (UI | NonUI)
    fn take_turn(&mut self, ix: Index<Self::Entity>) -> Option<EventData<Self::Event>>;

    fn handle_event(&mut self, ev: Self::Event, tree: &mut Self::EventTree) -> HandleResult;
}

/// Return value of [`tick`](fn.tick.html)
#[derive(Debug, Clone, Default)]
pub struct TickResult<S: System> {
    pub gui: Option<UiEventData>,
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
        let ev = match sys.take_turn(ix) {
            Some(ev) => ev,
            // next actor
            None => continue,
        };

        match ev {
            EventData::NonUI(ev) => {
                let res = sys.handle_event(ev, &mut tree);
                // TODO: handle result?
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
    UI(UiEventData),
}

/// Marker for UI event types
pub trait UiEvent {}

impl<E, T: UiEvent + 'static> From<T> for EventData<E> {
    fn from(_ev: T) -> Self {
        Self::UI(UiEventData {
            id: any::type_name::<T>().to_string(),
        })
    }
}

/// UI events are handled externally (by UI)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiEventData {
    id: String,
}

// TODO: use toy_arena slot type
type Slot = u32;

/// Utility for implementing [`System::next_actor`]
#[derive(Debug, Clone, Default)]
pub struct ActorSlot {
    slot: Slot,
}

impl ActorSlot {
    pub fn next<T>(&mut self, arena: &mut Arena<T>) -> Option<Index<T>> {
        self.slot %= arena.len() as Slot;
        let slot = self.slot;
        self.slot += 1;
        arena.get_by_slot(slot).map(|(ix, _data)| ix)
    }
}
