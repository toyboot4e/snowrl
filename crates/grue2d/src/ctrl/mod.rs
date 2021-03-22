/*!
Agents: objects with exclusive states that work on other data
*/

pub mod rogue;
pub use rogue::Rogue;

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
