/*!
Passive data structures. They don't work to each other; they just update themselves.

The point is to make flat data structure. Agents that work on data is put in another module so that
[`Data`] doesn't have to nest big structs.
*/

pub mod res;
pub mod world;

use snow2d::Ice;

use self::{res::Resources, world::World};

/// Passive data. They don't work to each other; they just update themselves.
#[derive(Debug)]
pub struct Data {
    /// Generic game context
    pub ice: Ice,
    /// Roguelike game world
    pub world: World,
    /// Data specific for SnowRL
    pub res: Resources,
}
