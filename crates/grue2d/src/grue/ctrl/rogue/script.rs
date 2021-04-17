/*!

Script integration

*/

use snow2d::utils::arena::Index;

use crate::grue::data::world::actor::Actor;

/// Refers to a specifc script
#[derive(Debug, Clone, Copy)]
pub enum ScriptRef {
    Interact {
        from: Index<Actor>,
        to: Index<Actor>,
    },
}
