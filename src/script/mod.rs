//! Script

pub mod talk;
pub mod txt;

use crate::turn::tick::ActorIx;

/// Refers to a specifc script
#[derive(Debug, Clone, Copy)]
pub enum ScriptRef {
    Interact { from: ActorIx, to: ActorIx },
}
