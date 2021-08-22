/*!
Event handling based on the chain-of-responsibilities pattern
*/

use std::{
    any::{self, Any, TypeId},
    collections::HashMap,
    fmt,
    marker::PhantomData,
};

use snow2d::utils::Derivative;

use crate::{
    ev::Event,
    sys::{HandleResult, System},
};

pub type DynEvent = Box<dyn Event>;

pub trait HubSystem: System {
    type Context;
}

/// Event handler
pub type HandlerT<T, C> = Box<dyn FnMut(&T, &mut C) -> Option<HandleResult>>;

/// Event handler in storage
type DynEventHandler<C> = Box<dyn FnMut(&DynEvent, &mut C) -> Option<HandleResult>>;

/// Event handling system based on chain-of-responsibilities.
#[derive(Debug, Default)]
pub struct EventHub<S: HubSystem> {
    handlers: CorHub<S>,
    _ty: PhantomData<fn() -> S>,
}

impl<S: HubSystem + 'static> EventHub<S> {
    pub fn build(mut mutator: impl FnMut(&mut EventHubBuilder<S>)) -> EventHub<S> {
        let mut builder = EventHubBuilder::default();
        (mutator)(&mut builder);
        builder.build_hub()
    }

    /// Dispatches event handlers one by one based on the chain-of-reponsibilities pattern
    pub fn handle(&mut self, ev: &DynEvent, hcx: &mut S::Context) -> HandleResult
    where
        <S as HubSystem>::Context: 'static,
    {
        // event type ID
        let id = ev.type_id();
        let cor = self
            .handlers
            .get_mut(id)
            .unwrap_or_else(|| panic!("Unable to find handler for event"));

        cor.handle(ev, hcx)
    }
}

#[derive(Debug)]
pub struct EventHubBuilder<S: HubSystem> {
    handlers: CorHub<S>,
    _ty: PhantomData<fn() -> S>,
}

impl<S: HubSystem> Default for EventHubBuilder<S> {
    fn default() -> Self {
        Self {
            handlers: Default::default(),
            _ty: PhantomData,
        }
    }
}

// TODO: set handler priority
impl<S: HubSystem> EventHubBuilder<S>
where
    <S as HubSystem>::Context: 'static,
{
    pub fn mutate(&mut self, mut mutator: impl FnMut(&mut Self)) -> &mut Self {
        (mutator)(self);
        self
    }

    /// Registers a new type of event with default handler
    pub fn ev_with<E: Event + 'static>(&mut self, hnd: HandlerT<E, S::Context>) -> &mut Self {
        self.ev::<E>().hnd(hnd)
    }

    /// Registers a new type of event
    pub fn ev<E: Event + 'static>(&mut self) -> &mut Self {
        self.handlers.register_event_type::<E>();
        self
    }

    /// Registers an event handler
    pub fn hnd<E: Event + 'static>(&mut self, hnd: HandlerT<E, S::Context>) -> &mut Self {
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

    pub fn build_hub(self) -> EventHub<S> {
        EventHub {
            handlers: self.handlers,
            _ty: PhantomData,
        }
    }
}

/// Set of [`Cor`] for each event T
#[derive(Derivative)]
#[derivative(Debug, Default)]
struct CorHub<S: HubSystem> {
    map: HashMap<TypeId, Cor<S::Context>>,
}

impl<S: HubSystem> CorHub<S>
where
    S::Context: 'static,
{
    pub fn register_event_type<E>(&mut self)
    where
        E: Event + 'static,
    {
        let dup = self
            .map
            .insert(TypeId::of::<E>(), Cor::<S::Context>::new::<E>());
        assert!(dup.is_none());
    }

    pub fn get(&self, id: TypeId) -> Option<&Cor<S::Context>> {
        match self.map.get(&id) {
            Some(cor) => {
                assert_eq!(cor.ev_ty, id);
                Some(cor)
            }
            None => None,
        }
    }

    pub fn get_mut(&mut self, id: TypeId) -> Option<&mut Cor<S::Context>> {
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

impl<C> fmt::Debug for Cor<C> {
    // TODO: better debug print
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cor<C>").field("ty", &self.ev_ty).finish()
    }
}

impl<C: 'static> Cor<C> {
    pub fn new<T: Event + 'static>() -> Self {
        Self {
            raw: Vec::new(),
            ev_ty: TypeId::of::<T>(),
        }
    }

    pub fn register_handler<T: Event + 'static>(&mut self, mut concrete_handler: HandlerT<T, C>) {
        assert_eq!(self.ev_ty, TypeId::of::<T>());

        // wrap the concrete-event handler
        let abstract_handler = move |abstract_event: &DynEvent, context: &mut C| {
            let concrete_event = abstract_event
                .as_any()
                .downcast_ref::<T>()
                .unwrap_or_else(|| {
                    unreachable!("Unable to cast event to type {}", any::type_name::<T>())
                });
            (concrete_handler)(concrete_event, context)
        };

        self.raw.push(Box::new(abstract_handler));
    }

    pub fn handle(&mut self, ev: &DynEvent, hcx: &mut C) -> HandleResult {
        for hnd in self.raw.iter_mut().rev() {
            if let Some(res) = (hnd)(ev, hcx) {
                return res;
            }
        }

        panic!("Unable to handle event {:?}", ev);
    }
}
