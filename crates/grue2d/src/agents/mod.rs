/*!
Object for the finite state machine
*/

mod renderer;
pub use renderer::*;

/// Objects with exclusive states that work on other data
#[derive(Debug)]
pub struct Agents {
    pub world_render: WorldRenderer,
}

impl Agents {
    pub fn new(screen_size: [u32; 2]) -> Self {
        Self {
            world_render: WorldRenderer::new(screen_size),
        }
    }
}
