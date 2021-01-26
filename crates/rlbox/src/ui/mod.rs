/*!

OMG, UI!

*/

pub mod anims;
pub mod node;

// TODO: scenes
pub mod stage;

use std::time::Duration;

use snow2d::gfx::draw::DrawApi;

use crate::utils::pool::Pool;

use self::{anims::Anim, node::Node};

/// Collection of sprites and animations
#[derive(Debug)]
pub struct Ui {
    pub nodes: Pool<Node>,
    pub anims: AnimPool,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            nodes: Pool::with_capacity(16),
            anims: AnimPool::default(),
        }
    }
}

impl Ui {
    pub fn update(&mut self, dt: Duration) {
        self.anims.tick(dt);
        self.anims.run(&mut self.nodes);
    }

    pub fn render(&mut self, draw: &mut impl DrawApi) {
        // TODO: sort nodes
        for node in self.nodes.items_mut() {
            println!("TODO: render node {:?}", node);
            node.render(draw);
        }
    }
}

#[derive(Debug)]
pub struct AnimPool(Pool<Anim>);

impl Default for AnimPool {
    fn default() -> Self {
        Self(Pool::with_capacity(24))
    }
}

impl std::ops::Deref for AnimPool {
    type Target = Pool<Anim>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AnimPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AnimPool {
    pub fn tick(&mut self, dt: Duration) {
        for anim in self.0.items_mut() {
            anim.tick(dt);
        }
    }

    pub fn run(&mut self, nodes: &mut Pool<Node>) {
        //
    }
}
