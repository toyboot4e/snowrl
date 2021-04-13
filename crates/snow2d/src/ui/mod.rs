/*!
UI or scene graph: container of sprites and animations
*/

pub mod anim;
pub mod anim_builder;
pub mod node;

use {glam::Mat4, std::time::Duration};

use crate::{
    utils::{
        arena::Arena,
        ez,
        pool::{Handle, Pool, Slot},
        Cheat,
    },
    Ice,
};

use self::{
    anim::{Anim, AnimImpl},
    anim_builder::AnimSeq,
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
    /// Used to retrieve target item
    slot: Slot,
    /// Used to sort entries
    order: Order,
}

pub struct SortedNodesMut<'a> {
    nodes: &'a mut Pool<Node>,
    orders: &'a [OrderEntry],
    order_pos: usize,
}

impl<'a> Iterator for SortedNodesMut<'a> {
    type Item = &'a mut Node;
    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.orders.get(self.order_pos)?.slot;
        self.order_pos += 1;

        let ptr = self
            .nodes
            .get_mut_by_slot(slot)
            .expect("unable to find node!") as *mut _;
        Some(unsafe { &mut *ptr })
    }
}

/// Nodes and animations
#[derive(Debug)]
pub struct Layer {
    pub nodes: NodePool,
    pub anims: AnimStorage,
    pub coord: CoordSystem,
    ord_buf: Vec<OrderEntry>,
}

impl Layer {
    pub fn new(coord: CoordSystem) -> Self {
        Self {
            nodes: NodePool::new(),
            anims: AnimStorage::new(),
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
            let nodes = Cheat::new(&self.nodes);
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

    fn sort_nodes(&mut self) {
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
    }

    pub fn nodes_mut_sorted<'a>(&'a mut self) -> SortedNodesMut<'a> {
        self.sort_nodes();
        SortedNodesMut {
            nodes: &mut self.nodes.pool,
            orders: &self.ord_buf,
            order_pos: 0,
        }
    }

    pub fn render(&mut self, ice: &mut Ice, cam_mat: Mat4) {
        let mut screen = ice
            .snow
            .screen()
            .transform(match self.coord {
                CoordSystem::Screen => None,
                CoordSystem::World => Some(cam_mat),
            })
            .build();

        // render
        for node in self.nodes_mut_sorted() {
            node.render(&mut screen);
        }
    }
}

/// Extended [`Pool`] for handling tree of nodes
#[derive(Debug)]
pub struct NodePool {
    pool: Pool<Node>,
}

impl std::ops::Deref for NodePool {
    type Target = Pool<Node>;
    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl std::ops::DerefMut for NodePool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pool
    }
}

impl NodePool {
    pub fn new() -> Self {
        Self {
            pool: Pool::with_capacity(16),
        }
    }

    pub fn attach_child(&mut self, parent_handle: &Handle<Node>, mut child: Node) -> Handle<Node> {
        child.parent = Some(parent_handle.clone());
        let child_handle = self.pool.add(child);
        self.pool[parent_handle]
            .children
            .push(child_handle.to_downgraded());
        child_handle
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DelayedAnim {
    delay: ez::LinearDt,
    is_first_tick: bool,
    anim: Anim,
}

impl DelayedAnim {
    pub fn new(delay: Duration, anim: Anim) -> Self {
        Self {
            delay: ez::LinearDt::new(delay.as_secs_f32()),
            is_first_tick: false,
            anim,
        }
    }
}

/// Extended [`Arena`] for animations
///
/// TODO: guarantee no duplicates exist
#[derive(Debug)]
pub struct AnimStorage {
    running: Arena<Anim>,
    delayed: Arena<DelayedAnim>,
}

impl AnimStorage {
    pub fn new() -> Self {
        Self {
            running: Arena::with_capacity(16),
            delayed: Arena::with_capacity(16),
        }
    }

    pub fn insert_seq(&mut self, seq: AnimSeq) {
        for anim in seq.anims {
            self.delayed.insert(anim);
        }
    }
}

impl std::ops::Deref for AnimStorage {
    type Target = Arena<Anim>;
    fn deref(&self) -> &Self::Target {
        &self.running
    }
}

impl std::ops::DerefMut for AnimStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.running
    }
}

impl AnimStorage {
    pub fn builder(&mut self) -> anim_builder::AnimBuilder {
        anim_builder::AnimBuilder::new(self)
    }

    /// Tick and apply animations. Remove finished animations
    pub fn update(&mut self, dt: Duration, nodes: &mut Pool<Node>) {
        // update `delayed` animations
        let mut starts = vec![];

        for (anim_ix, anim) in self.delayed.iter_mut() {
            // TODO: refactor with Timer, maybe in `ez`
            if anim.is_first_tick {
                anim.is_first_tick = false;
                // first tick: do end check BEFORE ticking
                if anim.delay.is_end() {
                    starts.push(anim_ix);
                } else {
                    anim.delay.tick(dt);
                }
            } else {
                anim.delay.tick(dt);
                // non-first tick: do end check AFTER ticking
                if anim.delay.is_end() {
                    starts.push(anim_ix);
                }
            }
        }

        for ix in starts {
            log::trace!("START");
            let mut delayed_anim = self.delayed.remove(ix).unwrap();
            delayed_anim.anim.set_active(true);
            self.running.insert(delayed_anim.anim);
        }

        // update `running` animations
        let mut removals = vec![];

        for (removal_ix, anim) in self.running.iter_mut() {
            // TODO: active property not needed?
            if !anim.is_active() {
                continue;
            }

            if anim.is_end() {
                removals.push(removal_ix);
                continue;
            }

            anim.tick(dt);
            anim.apply(nodes);
        }

        for ix in removals {
            self.running.remove(ix);
        }
    }
}
