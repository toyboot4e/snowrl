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
