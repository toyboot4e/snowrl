/*!
Event system
*/

pub mod hub;

use std::{fmt, ops};

use crate::sys::System;

use downcast_rs::Downcast;
use dyn_clone::DynClone;

/// Upcasted event
pub trait Event: fmt::Debug + DynClone + Downcast {}
downcast_rs::impl_downcast!(Event);
dyn_clone::clone_trait_object!(Event);

/// Roguelike game model
///
/// One `Model` can synced to another by applying changes over time.
pub trait Model {
    type Change;
    fn apply_change(&mut self, chg: &Self::Change);
}

/// Event builder + read-only access to model
#[derive(Debug)]
pub struct SystemArgs<S: System> {
    mdl: S::Model,
    tree: S::EventTree,
}

impl<S: System> SystemArgs<S> {
    pub fn new(mdl: S::Model) -> Self
    where
        S::EventTree: Default,
    {
        Self {
            mdl,
            tree: Default::default(),
        }
    }
}

impl<S: System> SystemArgs<S> {
    pub fn mdl(&self) -> &S::Model {
        &self.mdl
    }

    pub fn make_change(&mut self, chg: &<<S as System>::Model as self::Model>::Change) {
        self.mdl.apply_change(&chg);
    }

    pub fn retrieve(self) -> (S::Model, S::EventTree) {
        (self.mdl, self.tree)
    }

    pub fn tree(&mut self) -> &S::EventTree {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut S::EventTree {
        &mut self.tree
    }
}

impl<S: System> ops::Deref for SystemArgs<S> {
    type Target = S::Model;
    fn deref(&self) -> &Self::Target {
        &self.mdl
    }
}
