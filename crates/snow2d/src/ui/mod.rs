/*!
UI or scene graph: container of sprites and animations
*/

// TODO: Consider whether animation arena should be handled equally as user data
// The difference is that `ui::anim` doesn't need to refer to user data type to make changes

pub mod anim;
pub mod anim_builder;
pub mod node;

use std::time::Duration;

use crate::{
    gfx::{draw::*, RenderPass},
    utils::{
        arena::Arena,
        enum_dispatch, ez, inspect,
        pool::{Handle, Pool, Slot, WeakHandle},
        Cheat, Inspect,
    },
};

use self::{
    anim::*,
    anim_builder::AnimSeq,
    node::{Draw, DrawParams, Order},
};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Inspect)]
pub enum CoordSystem {
    /// Use fixed position to the screen
    Screen,
    /// Used world coordinates to render nodes. Follow camera automatically
    World,
}

/// Specifies coordinate system and z ordering
// TODO: maybe use `Rc`?
#[derive(Debug, Clone, Copy, PartialEq, Inspect)]
pub struct Layer {
    pub coord: CoordSystem,
    /// 0 to 1
    pub z_order: f32,
}

/// Visible object in a UI layer
#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Node {
    pub draw: Draw,
    /// Common geometry data
    pub params: DrawParams,
    /// Draw parameter calculated befre rendering
    pub(super) cache: DrawParams,
    /// Render layer: z ordering and coordinate system
    pub layer: Layer,
    /// Local rendering order in range [0, 1] (the higher, the latter drawn)
    pub z_order: Order,
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
            layer: Layer {
                coord: CoordSystem::Screen,
                z_order: 1.0,
            },
            z_order: 1.0,
            children: vec![],
            parent: None,
        }
    }
}

impl Node {
    pub fn global_z_order(&self) -> f32 {
        // FIXME:
        self.layer.z_order + self.z_order / 10.0
    }

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
#[derive(Debug, Clone, Inspect)]
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

/// Index of [`Anim`] in expected collection (i.e., generational arena)
pub type AnimIndex = crate::utils::arena::Index<Anim>;

/// Used for sorting nodes
#[derive(Debug)]
struct OrderEntry {
    /// Used to retrieve target item
    slot: Slot,
    /// Used to sort entries
    order: Order,
    coord: CoordSystem,
}

pub struct SortedNodesMut<'a> {
    nodes: &'a mut Pool<Node>,
    orders: &'a [OrderEntry],
    pos: usize,
}

impl<'a> Iterator for SortedNodesMut<'a> {
    type Item = &'a mut Node;
    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.orders.get(self.pos)?.slot;
        self.pos += 1;

        let ptr = self
            .nodes
            .get_mut_by_slot(slot)
            .expect("unable to find node!") as *mut _;
        Some(unsafe { &mut *ptr })
    }
}

/// Nodes and animations
#[derive(Debug, Inspect)]
pub struct Ui {
    pub nodes: NodePool,
    pub anims: AnimStorage,
    #[inspect(skip)]
    ord_buf: Vec<OrderEntry>,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            nodes: NodePool::new(),
            anims: AnimStorage::new(),
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
                order: node.global_z_order(),
                coord: node.layer.coord,
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
            pos: 0,
        }
    }

    /// FIXME: It basically ignores `node.layer.coord`.
    pub fn render_range(
        &mut self,
        range: impl std::ops::RangeBounds<f32>,
        pass: &mut RenderPass<'_>,
    ) {
        // TODO: more efficient rendering
        for node in &mut self
            .nodes_mut_sorted()
            .filter(|n| range.contains(&n.global_z_order()))
        {
            node.render(pass);
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
                    Inspect::inspect(x, ui, imgui::im_str!("{}", i).to_str());
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
