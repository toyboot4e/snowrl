/*!
Event system
*/

pub mod hub;
pub mod tree;

use std::fmt;

use dyn_clone::DynClone;

/// Upcasted event
pub trait Event: fmt::Debug + DynClone {}

dyn_clone::clone_trait_object!(Event);
