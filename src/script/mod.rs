//! Script

pub mod render;

use crate::turn::tick::ActorIx;

/// Refers to a specifc script
#[derive(Debug, Clone, Copy)]
pub enum ScriptRef {
    Interact { from: ActorIx, to: ActorIx },
}
