/*!
UI or scene graph: container of sprites and animations
*/

// TODO: Consider whether animation arena should be handled equally as user data
// The difference is that `ui::anim` doesn't need to refer to user data type to make changes

pub mod anim;
pub mod anim_builder;
pub mod node;

use {glam::Mat4, std::time::Duration};

use crate::{
    gfx::{draw::*, RenderPass},
    utils::{
        arena::Arena,
        enum_dispatch, ez, inspect,
        pool::{Handle, Pool, Slot, WeakHandle},
        Cheat, Inspect,
    },
    Ice,
};

use self::{
    anim::*,
    anim_builder::AnimSeq,
    node::{Draw, DrawParams, Order},
};

/// Coordinate used in a [`Layer`] (`Screen` | `World`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoordSystem {
    /// Use fixed position to the screen
    Screen,
    /// Used world coordinates to render nodes. Follow camera automatically
    World,
}

/// Visible object in a UI layer
#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Node {
    pub draw: Draw,
    /// Common geometry data
    pub params: DrawParams,
    /// Draw parameter calculated befre rendering
    pub(super) cache: DrawParams,
    /// Rendering order [0, 1] (the higher, the latter)
    pub order: Order,
    /// NOTE: Parents are alive if any children is alive
    pub(super) parent: Option<Handle<Node>>,
    pub(super) children: Vec<WeakHandle<Node>>,
    // TODO: dirty flag,
}

impl From<Draw> for Node {
    fn from(draw: Draw) -> Self {
        let params = DrawParams {
            size: match draw {
                // FIXME: parent box size. Node builder?
                Draw::None => [1.0, 1.0].into(),
                Draw::Sprite(ref x) => x.sub_tex_size_scaled().into(),
                Draw::NineSlice(ref x) => x.sub_tex_size_scaled().into(),
                // FIXME: measure text size?
                Draw::Text(ref _x) => [1.0, 1.0].into(),
            },
            ..Default::default()
        };

        Node {
            draw,
            params: params.clone(),
            cache: params.clone(),
            order: 1.0,
            children: vec![],
            parent: None,
        }
    }
}

impl Node {
    pub fn render(&mut self, pass: &mut RenderPass<'_>) {
        let params = &self.cache;
        match self.draw {
            Draw::Sprite(ref x) => {
                params.setup_quad(&mut pass.sprite(x));
            }
            Draw::NineSlice(ref x) => {
                params.setup_quad(&mut pass.sprite(x));
            }
            Draw::Text(ref x) => {
                // TODO: custom position
                pass.text(params.pos, &x.txt);
            }
            Draw::None => {}
        }
    }
}

/// One of [`AnimImpl`] impls
#[enum_dispatch(AnimImpl)]
#[derive(Debug, Clone)]
pub enum Anim {
    DynAnim,
    // tweens
    PosTween,
    XTween,
    YTween,
    SizeTween,
    ColorTween,
    AlphaTween,
    RotTween,
    // ParamsTween,
}

impl Inspect for Anim {
    fn inspect(&mut self, ui: &imgui::Ui, label: &str) {
        match self {
            Self::DynAnim(x) => x.inspect(ui, label),
            Self::PosTween(x) => x.inspect(ui, label),
            Self::XTween(x) => x.inspect(ui, label),
            Self::YTween(x) => x.inspect(ui, label),
            Self::SizeTween(x) => x.inspect(ui, label),
            Self::ColorTween(x) => x.inspect(ui, label),
            Self::AlphaTween(x) => x.inspect(ui, label),
            Self::RotTween(x) => x.inspect(ui, label),
        }
    }
}

/// Index of [`Anim`] in expected collection (i.e., generational arena)
pub type AnimIndex = crate::utils::arena::Index<Anim>;

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
#[derive(Debug, Inspect)]
pub struct Layer {
    pub nodes: NodePool,
    pub anims: AnimStorage,
    #[inspect(skip)]
    pub coord: CoordSystem,
    #[inspect(skip)]
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

        let _ = parent
            .as_mut()
            .children
            .drain_filter(|child_handle| {
                if let Some(child) = nodes.as_mut().get_mut(child_handle) {
                    Self::update_node_rec(nodes.clone(), child, Some(parent.clone()));
                    false // keep the valid child index
                } else {
                    true // drain the dangling child index
                }
            })
            .collect::<Vec<_>>();
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
#[derive(Debug, Inspect)]
#[inspect(in_place)]
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

    pub fn add_as_child(&mut self, parent_handle: &Handle<Node>, mut child: Node) -> Handle<Node> {
        child.parent = Some(parent_handle.clone());
        let child_handle = self.pool.add(child);
        self.pool[parent_handle]
            .children
            .push(child_handle.to_downgraded());
        child_handle
    }
}

#[derive(Debug, Clone, Inspect)]
pub(crate) struct DelayedAnim {
    delay: ez::LinearDt,
    is_first_tick: bool,
    #[inspect(skip)]
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

impl Inspect for AnimStorage {
    fn inspect(&mut self, ui: &imgui::Ui, label: &str) {
        inspect::nest(ui, label, || {
            inspect::nest(ui, "running", || {
                for (i, (_index, x)) in self.running.iter_mut().enumerate() {
                    x.inspect(ui, imgui::im_str!("{}", i).to_str());
                }
            });

            inspect::inspect_seq(self.delayed.iter_mut().map(|(_i, x)| x), ui, "delayed");
        });
    }
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

    pub fn insert_delayed(&mut self, delay: Duration, anim: Anim) {
        self.delayed.insert(DelayedAnim::new(delay, anim));
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
    /// Tick and apply animations. Remove finished animations
    pub fn update(&mut self, dt: Duration, nodes: &mut Pool<Node>) {
        // update `delayed` animations
        let new_start_anims = self.delayed.drain_filter(|anim| {
            // TODO: refactor with Timer, maybe in `ez`
            if anim.is_first_tick {
                anim.is_first_tick = false;

                // first tick: do end check BEFORE ticking
                if anim.delay.is_end() {
                    true // drain
                } else {
                    anim.delay.tick(dt);
                    false
                }
            } else {
                // non-first tick: do end check AFTER ticking
                anim.delay.tick(dt);
                anim.delay.is_end()
            }
        });

        for mut anim in new_start_anims.map(|(_ix, delayed)| delayed.anim) {
            anim.set_active(true);
            self.running.insert(anim);
        }

        // update `running` animations
        let _ = self
            .running
            .drain_filter(|anim| {
                // TODO: active property not needed?
                if !anim.is_active() {
                    return false;
                }

                if anim.is_end() {
                    return true; // drain
                }

                anim.tick(dt);
                anim.apply(nodes);
                false
            })
            .collect::<Vec<_>>();
    }
}
