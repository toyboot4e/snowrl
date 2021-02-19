/*!
User interface
*/

pub mod anim;
pub mod builder;
pub mod node;

// TODO: scenes
pub mod stage;

use std::time::Duration;

use snow2d::{gfx::PassConfig, Ice};

use crate::utils::{
    arena::{Arena, Index},
    pool::Pool,
};

use self::{
    anim::{Anim, AnimImpl},
    node::Node,
};

/// Collection of layers
#[derive(Debug, Default)]
pub struct Ui {
    pub layers: Arena<Layer>,
}

impl Ui {
    pub fn update(&mut self, dt: Duration) {
        for (_ix, layer) in &mut self.layers {
            layer.update(dt);
        }
    }
}

/// Sprites and animations
#[derive(Debug)]
pub struct Layer {
    pub nodes: Pool<Node>,
    pub anims: AnimArena,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            nodes: Pool::with_capacity(16),
            anims: AnimArena(Arena::with_capacity(16)),
        }
    }
}

impl Layer {
    fn update(&mut self, dt: Duration) {
        self.anims.update(dt, &mut self.nodes);
        self.nodes.sync_refcounts();
    }

    pub fn render(&mut self, ice: &mut Ice) {
        // TODO: sort nodes
        let mut screen = ice.rdr.screen(PassConfig::default());
        for node in self.nodes.iter_mut() {
            node.render(&mut screen);
        }
    }
}

#[derive(Debug)]
pub struct AnimArena(Arena<Anim>);

impl std::ops::Deref for AnimArena {
    type Target = Arena<Anim>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AnimArena {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AnimArena {
    pub fn builder(&mut self) -> builder::AnimBuilder {
        builder::AnimBuilder::new(self)
    }

    /// Ticks and applies tweens
    pub fn update(&mut self, dt: Duration, nodes: &mut Pool<Node>) {
        let mut removals = vec![];

        for (ix, a) in self.0.iter_mut() {
            if a.is_end() {
                removals.push(ix);
                continue;
            }

            a.tick(dt);
            a.apply(nodes);
        }

        for ix in removals {
            // log::trace!("remove animation at {:?}", ix);
            self.0.remove(ix);
        }
    }
}
