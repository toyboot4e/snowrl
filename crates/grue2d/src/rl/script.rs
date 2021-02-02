/*!

Script integration

*/

use rlbox::utils::arena::Index;

use crate::rl::world::actor::Actor;

/// Refers to a specifc script
#[derive(Debug, Clone, Copy)]
pub enum ScriptRef {
    Interact {
        from: Index<Actor>,
        to: Index<Actor>,
    },
}
