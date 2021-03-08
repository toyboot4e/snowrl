/*!
User interface (sprites and animations)
*/

pub mod anim;
pub mod anim_builder;
pub mod node;

use {glam::Mat4, std::time::Duration};

use crate::{
    gfx::PassConfig,
    utils::{
        arena::Arena,
        cheat::Cheat,
        pool::{Pool, Slot},
    },
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

    pub fn update(&mut self, dt: Duration) {
        // apply animations
        self.anims.update(dt, &mut self.nodes);

        self.nodes.sync_refcounts();

        // calculate geometry
        unsafe {
            let mut nodes = Cheat::new(&mut self.nodes);
            for node in nodes.iter_mut() {
                // update cache
                self.update_node_rec(node, None);
            }
        }
        // TODO: sort nodes
    }

    unsafe fn update_node_rec(&mut self, child: &mut Node, parent: Option<Slot>) {
        child.cache = child.params.clone();
        if let Some(parent) = parent {
            let parent = self.nodes.get_by_slot_mut(parent).unwrap();
            parent.cache.transform_mut(&mut child.cache);
        }
    }

    pub fn render(&mut self, ice: &mut Ice, cam_mat: Mat4) {
        let mut screen = ice.snow.screen(PassConfig {
            tfm: match self.coord {
                CoordSystem::Screen => None,
                CoordSystem::World => Some(cam_mat),
            },
            ..Default::default()
        });

        for node in &mut self.nodes {
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
    pub fn builder(&mut self) -> anim_builder::AnimBuilder {
        anim_builder::AnimBuilder::new(self)
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
