/*!
User interface (sprites and animations)
*/

pub mod anim;
pub mod builder;
pub mod node;

use {glam::Mat4, std::time::Duration};

use crate::{
    gfx::PassConfig,
    utils::{arena::Arena, pool::Pool},
    Ice,
};

use self::{
    anim::{Anim, AnimImpl},
    node::Node,
};

/// Coordinate used in a [`Layer`] (`Screen` | `World`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoordSystem {
    /// Use fixed position to the screen
    Screen,
    /// Used world coordinates to render nodes. Follow camera automatically
    World,
}

/// UI scene composed of layers
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

/// Nodes and animations
#[derive(Debug)]
pub struct Layer {
    pub nodes: NodePool,
    pub anims: AnimArena,
    pub coord: CoordSystem,
}

impl Layer {
    pub fn new(coord: CoordSystem) -> Self {
        Self {
            nodes: Pool::with_capacity(16),
            anims: AnimArena(Arena::with_capacity(16)),
            coord,
        }
    }

    fn update(&mut self, dt: Duration) {
        self.anims.update(dt, &mut self.nodes);
        self.nodes.sync_refcounts();
    }

    pub fn render(&mut self, ice: &mut Ice, cam_mat: Mat4) {
        let mut screen = ice.rdr.screen(PassConfig {
            tfm: match self.coord {
                CoordSystem::Screen => None,
                CoordSystem::World => Some(cam_mat),
            },
            ..Default::default()
        });

        // TODO: sort nodes
        for node in self.nodes.iter_mut() {
            node.render(&mut screen);
        }
    }
}

/// [`Pool`] of nodes
pub type NodePool = Pool<Node>;

/// [`Arena`] of animations
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
