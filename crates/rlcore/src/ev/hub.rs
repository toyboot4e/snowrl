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
pub type HandlerT<E: Event, C> = Box<dyn FnMut(&E, &mut C) -> Option<HandleResult>>;

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
        let cor: &mut Cor<HandlerT<E, S::Context>> = self.handlers.get_mut().unwrap_or_else(|| {
            panic!(
                "Unable to find handler of
    event type {}",
                any::type_name::<E>()
            )
        });

        cor.handle(ev, hcx)
    }

    pub fn handle_any(&mut self, any: AnyEv, hcx: &mut S::Context) {
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

impl<S: HubSystem> EventHubBuilder<S> {
    /// Register new type of event
    pub fn ev<E: Event + 'static>(&mut self) -> &mut Self {
        let dup = self
            .handlers
            .map
            .insert(TypeId::of::<E>(), Box::new(<Cor<E> as Default>::default()));
        assert!(dup.is_none());
        self
    }

    // pub fn ui_ev

    /// Register an event handler
    pub fn hnd<E: Event + 'static>(&mut self, hnd: HandlerT<E, S::Context>) -> &mut Self
    where
        <S as HubSystem>::Context: 'static,
    {
        let handlers: &mut Cor<HandlerT<E, S::Context>> = self
            .handlers
            .get_mut()
            .unwrap_or_else(|| panic!("Unable to find event of type {}", any::type_name::<E>(),));

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
    pub fn get<H: 'static>(&self) -> Option<&Cor<H>> {
        self.map
            .get(&TypeId::of::<H>())
            .map(|any| any.downcast_ref().unwrap())
    }

    pub fn get_mut<H: 'static>(&mut self) -> Option<&mut Cor<H>> {
        self.map
            .get_mut(&TypeId::of::<H>())
            .map(|any| any.downcast_mut().unwrap())
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
