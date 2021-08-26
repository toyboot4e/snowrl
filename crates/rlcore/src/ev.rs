/*!
Event system
*/

pub mod hub;
pub mod tree;

use std::fmt;

use downcast_rs::Downcast;
use dyn_clone::DynClone;

/// Upcasted event
pub trait Event: fmt::Debug + DynClone + Downcast {}

downcast_rs::impl_downcast!(Event);

dyn_clone::clone_trait_object!(Event);

use std::ops;

use crate::{ev::tree::EventBuilder, sys::System};

/// Framework
pub trait HubSystem: System {
    type Args;
}

pub trait Model {
    type Change;
    fn apply_change(&mut self, chg: &Self::Change);
}

/// Event builder + read-only access to model
#[derive(Debug)]
pub struct SystemArgs<M> {
    builder: EventBuilder,
    model: M,
}

impl<M: Model> SystemArgs<M> {
    pub fn make_change(&mut self, chg: &M::Change) {
        self.model.apply_change(chg);
    }
}

impl<M> SystemArgs<M> {
    pub fn new(model: M) -> Self {
        Self {
            builder: EventBuilder::default(),
            model,
        }
    }

    pub fn retrieve(self) -> (M, EventBuilder) {
        (self.model, self.builder)
    }

    pub fn model(&self) -> &M {
        &self.model
    }

    pub fn tree(&mut self) -> &EventBuilder {
        &self.builder
    }

    pub fn tree_mut(&mut self) -> &mut EventBuilder {
        &mut self.builder
    }
}

impl<M> ops::Deref for SystemArgs<M> {
    type Target = M;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}
