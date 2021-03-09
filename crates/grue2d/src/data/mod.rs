/*!

The roguelike framework for SnowRL

*/

pub mod resources;
pub mod rogue;
pub mod world;

use snow2d::Ice;

use self::{resources::Resources, rogue::Rogue, world::World};

/// The global data structure
#[derive(Debug)]
pub struct Data {
    pub ice: Ice,
    pub world: World,
    pub res: Resources,
    pub rogue: Rogue,
}
