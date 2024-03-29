/*!
Fixed set of components of [`GrueRl`](crate::GrueRl) controlled by [`fsm`](crate::fsm)
*/

pub mod agents;
pub mod ctrl;
pub mod data;

pub mod cfg;

use snow2d::{gfx::GameClock, Ice};

use self::{
    agents::WorldRenderer,
    cfg::GameConfig,
    ctrl::Rogue,
    data::{res::Resources, world::World},
};

/// Passive data to be operated on
#[derive(Debug)]
pub struct Data {
    /// Generic game context
    pub ice: Ice,
    /// Roguelike game world
    pub world: World,
    /// Data specific for SnowRL
    pub res: Resources,
    /// How we run the game
    pub cfg: GameConfig,
}

/// States to control the GUI roguelike game
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

/// Objects with exclusive states that work on / look into other [`Data`]
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
