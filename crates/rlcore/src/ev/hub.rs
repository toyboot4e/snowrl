/*!
Event handling based on the chain-of-responsibilities pattern
*/

// TODO: separate change from event
// TODO: add overridable logic functions

use std::{
    any::{self, Any, TypeId},
    collections::HashMap,
    fmt,
    marker::PhantomData,
};

use snow2d::utils::Derivative;

use crate::ev::{Event, SystemArgs};

pub type DynEvent = Box<dyn Event>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HandleResult {
    Handled,
    NotHandled,
}

/// Event handler
pub type HandlerT<T, C> = Box<dyn FnMut(&T, &mut C) -> HandleResult>;

/// Event handler in storage
type DynEventHandler<C> = Box<dyn FnMut(&DynEvent, &mut C) -> HandleResult>;

/// Event handling system based on chain-of-responsibilities.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct EventHub<M> {
    handlers: CorHub<M>,
    _ty: PhantomData<fn() -> M>,
}

impl<M> Default for EventHub<M> {
    fn default() -> Self {
        Self {
            handlers: CorHub::default(),
            _ty: PhantomData,
        }
    }
}

impl<M: 'static> EventHub<M> {
    pub fn build(mut mutator: impl FnMut(&mut EventHubBuilder<M>)) -> EventHub<M> {
        let mut builder = EventHubBuilder::default();
        (mutator)(&mut builder);
        builder.build_hub()
    }

    /// Dispatches event handlers one by one based on the chain-of-reponsibilities pattern
    pub fn handle(&mut self, ev: &DynEvent, args: &mut SystemArgs<M>) {
        // event type ID
        let id = ev.type_id();
        let cor = self
            .handlers
            .get_mut(id)
            .unwrap_or_else(|| panic!("Unable to find handler for event"));

        cor.handle(ev, args)
    }
}

pub struct EventHubBuilder<M> {
    handlers: CorHub<M>,
    _ty: PhantomData<fn() -> M>,
}

impl<M> Default for EventHubBuilder<M> {
    fn default() -> Self {
        Self {
            handlers: CorHub::default(),
            _ty: PhantomData,
        }
    }
}

impl<M: 'static> EventHubBuilder<M> {
    pub fn mutate(&mut self, mut mutator: impl FnMut(&mut Self)) -> &mut Self {
        (mutator)(self);
        self
    }

    /// Registers a new type of event with default handler
    pub fn ev_with<E: Event + 'static>(&mut self, hnd: HandlerT<E, SystemArgs<M>>) -> &mut Self {
        self.ev::<E>().hnd(hnd)
    }

    /// Registers a new type of event
    pub fn ev<E: Event + 'static>(&mut self) -> &mut Self {
        self.handlers.register_event_type::<E>();
        self
    }

    /// Registers an event handler
    pub fn hnd<E: Event + 'static>(&mut self, hnd: HandlerT<E, SystemArgs<M>>) -> &mut Self {
        let id = TypeId::of::<E>();

        let handlers = self.handlers.get_mut(id).unwrap_or_else(|| {
            panic!(
                "Unable to find handler for event of type {}",
                any::type_name::<E>(),
            )
        });

        // TODO: ensure no duplicate handlers exist
        handlers.register_handler(hnd);

        self
    }

    pub fn build_hub(self) -> EventHub<M> {
        EventHub {
            handlers: self.handlers,
            _ty: PhantomData,
        }
    }
}

/// Set of [`Cor`] for each event T
// #[derive(Debug, Default)]
#[derive(Derivative)]
#[derivative(Debug)]
struct CorHub<M> {
    map: HashMap<TypeId, Cor<SystemArgs<M>>>,
}

impl<M> Default for CorHub<M> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<M: 'static> CorHub<M> {
    pub fn register_event_type<E>(&mut self)
    where
        E: Event + 'static,
    {
        let dup = self
            .map
            .insert(TypeId::of::<E>(), Cor::<SystemArgs<M>>::new::<E>());
        assert!(dup.is_none());
    }

    pub fn get(&self, id: TypeId) -> Option<&Cor<SystemArgs<M>>> {
        match self.map.get(&id) {
            Some(cor) => {
                assert_eq!(cor.ev_ty, id);
                Some(cor)
            }
            None => None,
        }
    }

    pub fn get_mut(&mut self, id: TypeId) -> Option<&mut Cor<SystemArgs<M>>> {
        match self.map.get_mut(&id) {
            Some(cor) => {
                assert_eq!(cor.ev_ty, id);
                Some(cor)
            }
            None => None,
        }
    }
}

/// Set of event handlers for speccific event types (binded at runtime)
struct Cor<C> {
    raw: Vec<DynEventHandler<C>>,
    /// Interested event type (determined dynamically)
    ev_ty: TypeId,
}

impl<A> fmt::Debug for Cor<A> {
    // TODO: better debug print
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cor<C>").field("ty", &self.ev_ty).finish()
    }
}

impl<A: 'static> Cor<A> {
    pub fn new<T: Event + 'static>() -> Self {
        Self {
            raw: Vec::new(),
            ev_ty: TypeId::of::<T>(),
        }
    }

    pub fn register_handler<T: Event + 'static>(&mut self, mut concrete_handler: HandlerT<T, A>) {
        assert_eq!(self.ev_ty, TypeId::of::<T>());

        // wrap the concrete-event handler
        let abstract_handler = move |abstract_event: &DynEvent, args: &mut A| {
            let concrete_event = abstract_event
                .as_any()
                .downcast_ref::<T>()
                .unwrap_or_else(|| {
                    unreachable!("Unable to cast event to type {}", any::type_name::<T>())
                });
            (concrete_handler)(concrete_event, args)
        };

        self.raw.push(Box::new(abstract_handler));
    }

    pub fn handle(&mut self, ev: &DynEvent, hcx: &mut A) {
        for hnd in self.raw.iter_mut().rev() {
            match (hnd)(ev, hcx) {
                HandleResult::Handled => return,
                HandleResult::NotHandled => {}
            }
        }

        panic!("Unable to handle event {:?}", ev);
    }
}
