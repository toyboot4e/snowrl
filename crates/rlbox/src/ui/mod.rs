/*!

User interface

*/

pub mod anims;
pub mod node;

// TODO: scenes
pub mod stage;

use std::time::Duration;

use snow2d::gfx::draw::DrawApi;

use crate::utils::{arena::Arena, pool::Pool};

use self::{
    anims::{Anim, AnimImpl},
    node::Node,
};

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
        self.anims.update(dt, &mut self.nodes);
    }

    pub fn render(&mut self, draw: &mut impl DrawApi) {
        // TODO: sort nodes
        for node in self.nodes.items_mut() {
            node.render(draw);
        }
    }
}

#[derive(Debug)]
pub struct AnimPool(Arena<Anim>);

impl Default for AnimPool {
    fn default() -> Self {
        Self(Arena::with_capacity(16))
    }
}

impl std::ops::Deref for AnimPool {
    type Target = Arena<Anim>;
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
    /// Ticks and applies tweens
    pub fn update(&mut self, dt: Duration, nodes: &mut Pool<Node>) {
        for (_ix, a) in self.0.iter_mut() {
            a.tick(dt);
            match a {
                Anim::Seq(_x) => unimplemented!(),
                Anim::Parallel(_x) => unimplemented!(),
                Anim::PosTween(x) => {
                    let n = &mut nodes[&x.node];
                    n.geom.pos = x.tween.get();
                }
            }
        }
    }
}
