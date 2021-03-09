/*!
Agents: objects with exclusive state
*/

pub mod renderer;
use renderer::WorldRenderer;

#[derive(Debug)]
pub struct Agents {
    pub world_render: WorldRenderer,
}

impl Agents {
    pub fn new() -> Self {
        Self {
            world_render: WorldRenderer::default(),
        }
    }
}
