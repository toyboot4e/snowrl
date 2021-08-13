/*!
Event handling based on the chain-of-responsibilities pattern
*/

use std::{
    any::{self, Any, TypeId},
    collections::HashMap,
    fmt,
    marker::PhantomData,
};

use crate::{
    ev::Event,
    sys::{HandleResult, System},
};

pub type AnyEv = Box<dyn Event>;

pub trait HubSystem: System {
    type Context;
}

/// Event handler
pub type HandlerT<E, C> = Box<dyn FnMut(&E, &mut C) -> Option<HandleResult>>;

/// Event handling system based on chain-of-responsibilities.
#[derive(Debug, Default)]
pub struct EventHub<S> {
    handlers: CorHub,
    _ty: PhantomData<S>,
}

impl<S: HubSystem> EventHub<S> {
    pub fn builder() -> EventHubBuilder<S> {
        EventHubBuilder::default()
    }

    pub fn handle_t<E: Event + 'static>(&mut self, ev: &E, hcx: &mut S::Context) -> HandleResult
    where
        <S as HubSystem>::Context: 'static,
    {
        let cor = self.handlers.get_mut::<E, S>().unwrap_or_else(|| {
            panic!(
                "Unable to find handler of
    event type {}",
                any::type_name::<E>()
            )
        });

        cor.handle(ev, hcx)
    }

    pub fn handle_any(&mut self, _any: AnyEv, _hcx: &mut S::Context) {
        todo!()
    }
}

#[derive(Debug)]
pub struct EventHubBuilder<S: HubSystem> {
    handlers: CorHub,
    _ty: PhantomData<S>,
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
    /// Registers a new type of event with default handler
    pub fn ev_with<E: Event + 'static>(&mut self, hnd: HandlerT<E, S::Context>) -> &mut Self {
        self.ev::<E>().hnd(hnd)
    }

    /// Registers a new type of event
    pub fn ev<E: Event + 'static>(&mut self) -> &mut Self {
        self.handlers.register::<E, S>();
        self
    }

    /// Registers an event handler
    pub fn hnd<E: Event + 'static>(&mut self, hnd: HandlerT<E, S::Context>) -> &mut Self {
        let handlers = self.handlers.get_mut::<E, S>().unwrap_or_else(|| {
            panic!(
                "Unable to find handler for event of type {}",
                any::type_name::<E>(),
            )
        });

        // TODO: ensure no duplicate handlers exist
        handlers.raw.push(hnd);

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
#[derive(Debug, Default)]
struct CorHub {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl CorHub {
    fn handler_id<E, S>() -> TypeId
    where
        E: 'static,
        S: HubSystem,
        S::Context: 'static,
    {
        TypeId::of::<HandlerT<E, S::Context>>()
    }

    fn register<E, S>(&mut self)
    where
        E: 'static,
        S: HubSystem,
        S::Context: 'static,
    {
        let id = Self::handler_id::<E, S>();
        let dup = self.map.insert(
            id,
            Box::new(<Cor<HandlerT<E, S::Context>> as Default>::default()),
        );
        assert!(dup.is_none());
    }

    // FIXME: use TypId as arguent, not generic methods

    pub fn get<E, S>(&self) -> Option<&Cor<HandlerT<E, S::Context>>>
    where
        E: 'static,
        S: HubSystem,
        S::Context: 'static,
    {
        let id = Self::handler_id::<E, S>();
        self.map.get(&id).map(|any| {
            any.downcast_ref().unwrap_or_else(|| {
                panic!("Unable to cast CoR for event `{}`", any::type_name::<E>())
            })
        })
    }

    pub fn get_mut<E, S>(&mut self) -> Option<&mut Cor<HandlerT<E, S::Context>>>
    where
        E: 'static,
        S: HubSystem,
        S::Context: 'static,
    {
        let id = Self::handler_id::<E, S>();
        self.map.get_mut(&id).map(|any| {
            any.downcast_mut().unwrap_or_else(|| {
                panic!("Unable to cast CoR for event `{}`", any::type_name::<E>())
            })
        })
    }
}

/// Set of event handlers based on chain-of-responsibilities
struct Cor<H> {
    raw: Vec<H>,
    dirty: bool,
}

impl<E: Event + 'static + fmt::Debug> fmt::Debug for Cor<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cor<T>")
            // .field("handlers", &self.handlers)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<T> Default for Cor<T> {
    fn default() -> Self {
        Self {
            raw: Default::default(),
            dirty: Default::default(),
        }
    }
}

impl<E: Event + 'static, Context> Cor<HandlerT<E, Context>> {
    pub fn handle(&mut self, ev: &E, hcx: &mut Context) -> HandleResult {
        for hnd in self.raw.iter_mut().rev() {
            if let Some(res) = (hnd)(ev, hcx) {
                return res;
            }
        }

        panic!("Unable to handle event {:?}", ev);
    }
}
