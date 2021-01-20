//! Script

pub mod render;

use crate::turn::tick::ActorIx;

#[derive(Debug, Clone, Copy)]
pub enum ScriptRef {
    Interact { from: ActorIx, to: ActorIx },
}
