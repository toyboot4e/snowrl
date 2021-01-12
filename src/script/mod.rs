//! Script

use crate::turn::tick::ActorIx;

// pub trait Script {
// }

// pub trait Interactable {
//     fn on_interact(&mut self) -> Option<Box<dyn Script>>;
// }

#[derive(Debug, Clone, Copy)]
pub enum ScriptRef {
    Interact { from: ActorIx, to: ActorIx },
}
