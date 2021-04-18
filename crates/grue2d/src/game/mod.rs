/*!
Game data controled by `fsm`
*/

pub mod agents;
pub mod ctrl;
pub mod data;

use snow2d::{gfx::GameClock, Ice};

use self::{
    agents::WorldRenderer,
    ctrl::Rogue,
    data::{res::Resources, world::World},
};

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

/// States to control the roguelike data
#[derive(Debug)]
pub struct Control {
    /// State for playing roguelike game
    pub rogue: Rogue,
}

impl Control {
    pub fn new() -> Self {
        Self {
            rogue: Rogue::new(),
        }
    }
}

/// Objects with exclusive states that work on other data
#[derive(Debug)]
pub struct Agents {
    pub world_render: WorldRenderer,
}

impl Agents {
    pub fn new(screen_size: [u32; 2], clock: &GameClock) -> Self {
        Self {
            world_render: WorldRenderer::new(screen_size, clock),
        }
    }
}
