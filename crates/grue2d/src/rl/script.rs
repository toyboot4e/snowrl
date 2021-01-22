/*!

Script integration

*/

use crate::rl::turn::tick::ActorIx;

/// Refers to a specifc script
#[derive(Debug, Clone, Copy)]
pub enum ScriptRef {
    Interact { from: ActorIx, to: ActorIx },
}
