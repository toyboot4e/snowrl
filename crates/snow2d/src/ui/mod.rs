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
        pool::{Handle, Pool, Slot},
        Cheat,
    },
    Ice,
};

use self::{
    anim::{Anim, AnimImpl},
    node::{Node, Order},
};

/// Coordinate used in a [`Layer`] (`Screen` | `World`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoordSystem {
    /// Use fixed position to the screen
    Screen,
    /// Used world coordinates to render nodes. Follow camera automatically
    World,
}

/// Used for sorting nodes
#[derive(Debug)]
struct OrderEntry {
    slot: Slot,
    order: Order,
}

/// Nodes and animations
#[derive(Debug)]
pub struct Layer {
    pub nodes: NodePool,
    pub anims: AnimArena,
    pub coord: CoordSystem,
    ord_buf: Vec<OrderEntry>,
}

impl Layer {
    pub fn new(coord: CoordSystem) -> Self {
        Self {
            nodes: NodePool::new(),
            anims: AnimArena(Arena::with_capacity(16)),
            coord,
            ord_buf: Vec::with_capacity(16),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        // tick and apply animations. remove finished animations
        self.anims.update(dt, &mut self.nodes);

        // remove unreferenced nodes
        self.nodes.sync_refcounts();

        // calculate geometry
        unsafe {
            let nodes = Cheat::new(&mut self.nodes);
            for node in nodes.as_mut().iter_mut().filter(|n| n.parent.is_none()) {
                // update cache
                Self::update_node_rec(nodes.clone(), node, None);
            }
        }
    }

    unsafe fn update_node_rec(
        nodes: Cheat<NodePool>,
        child: &mut Node,
        parent: Option<Cheat<Node>>,
    ) {
        // load animated paramaters to cache
        child.cache = child.params.clone();

        // apply transformation to this node
        if let Some(parent) = parent {
            parent.cache.transform_mut(&mut child.cache);
        }

        // apply transformation to children
        let parent = Cheat::new(child);
        for child_slot in &parent.as_mut().children {
            let child = nodes.as_mut().get_mut(child_slot).unwrap();
            Self::update_node_rec(nodes.clone(), child, Some(parent.clone()));
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

        // sort nodes
        self.ord_buf.clear();
        for (slot, node) in self.nodes.enumerate_items() {
            self.ord_buf.push(OrderEntry {
                slot,
                order: node.order,
            });
        }

        self.ord_buf.sort_by(|e1, e2| {
            e1.order
                .partial_cmp(&e2.order)
                .expect("NAN found in ordering value of node")
        });

        // render
        for entry in &self.ord_buf {
            self.nodes
                .get_mut_by_slot(entry.slot)
                .unwrap()
                .render(&mut screen);
        }
    }
}

/// [`Pool`] of nodes

/// [`Arena`] of animations
#[derive(Debug)]
pub struct NodePool {
    nodes: Pool<Node>,
}

impl std::ops::Deref for NodePool {
    type Target = Pool<Node>;
    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl std::ops::DerefMut for NodePool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}

impl NodePool {
    pub fn new() -> Self {
        Self {
            nodes: Pool::with_capacity(16),
        }
    }

    pub fn attach_child(&mut self, parent_handle: &Handle<Node>, mut child: Node) -> Handle<Node> {
        child.parent = Some(parent_handle.clone());
        let child_handle = self.nodes.add(child);
        self.nodes[parent_handle]
            .children
            .push(child_handle.to_downgraded());
        child_handle
    }
}

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

    /// Tick and apply animations. Remove finished animations
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
