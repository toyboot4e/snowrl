/*!
Framework of SnowRL based on [`rlbox`]
*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub extern crate rlbox;

pub mod fsm;
pub mod render;
pub mod rl;

mod resources;
pub use resources::*;

use {
    rokol::{
        app::{self as ra, RApp},
        gfx as rg, Rokol,
    },
    snow2d::{gfx::geom2d::Vec2f, ui::Ui, Ice},
};

use crate::{
    fsm::*,
    render::WorldRenderer,
    rl::{script::ScriptRef, turn::anim::AnimPlayer, world::World},
};

pub extern crate hot_crate;
pub trait Plugin: std::fmt::Debug {}

/// Runs [`RApp`], which provides 60 FPS fixed-timestep game loop
pub fn run<App: RApp, AppConstructor: FnOnce(Rokol) -> App>(
    rokol: Rokol,
    constructor: AppConstructor,
) -> rokol::Result {
    rokol.run(&mut Runner {
        init_rokol: Some(rokol.clone()),
        init: Some(constructor),
        x: None,
    })
}

/// Creates [`RApp`] _after_ creating `rokol::gfx` contexts
struct Runner<T: RApp, F: FnOnce(Rokol) -> T> {
    init_rokol: Option<Rokol>,
    /// Use `Option` for lazy initialization
    init: Option<F>,
    /// Use `Option` for lazy initialization
    x: Option<T>,
}

impl<T: RApp, F: FnOnce(Rokol) -> T> rokol::app::RApp for Runner<T, F> {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());
        let f = self.init.take().unwrap();
        self.x = Some(f(self.init_rokol.take().unwrap()));
    }

    fn event(&mut self, ev: &ra::Event) {
        if let Some(x) = self.x.as_mut() {
            x.event(ev);
        }
    }

    fn frame(&mut self) {
        if let Some(x) = self.x.as_mut() {
            x.frame();
        }
    }
}

/// Runs a [`Fsm`] with [`Global`]
///
/// [`Fsm`] fsm::Fsm
/// [`Global`] fsm::Global
#[derive(Debug)]
pub struct GlueRl {
    /// Game context
    pub gl: Global,
    /// Game control based on stack-based state machine
    pub fsm: Fsm,
}

impl GlueRl {
    pub fn new(gl: Global, fsm: Fsm) -> Self {
        Self { gl, fsm }
    }
}

/// Thread-local global game states
///
/// TODO: consider using `Global<T>` for additional contexts
#[derive(Debug)]
pub struct Global {
    // SnowRL-only context
    pub world: World,
    pub world_render: WorldRenderer,
    pub vi: VInput,
    pub fonts: Fonts,
    // generic game context
    pub ice: Ice,
    pub ui: Ui,
    // game data
    pub anims: AnimPlayer,
    // TODO: extract it to user data
    pub script_to_play: Option<ScriptRef>,
}

/// Lifecycle of the contexts
impl Global {
    pub fn event(&mut self, ev: &ra::Event) {
        self.ice.event(ev);
    }

    /// Ticks input/graphics times
    //
    /// Called before updating the FSM (game state).
    pub fn pre_update(&mut self) {
        self.ice.pre_update();
        self.vi.update(&self.ice.input, self.ice.dt);
        self.world.update(&mut self.ice);
    }

    /// Updates buffers and ticks UI state
    //
    /// Called after updating the FSM (game state).
    pub fn post_update(&mut self) {
        // shadow
        // TODO: don't hard code player detection
        let player = &self.world.entities.get_by_slot(0).unwrap().1;
        self.world
            .shadow
            .post_update(self.ice.dt, &self.world.map.rlmap, player.pos);

        // camera
        let player_pos = player.img.pos_world_centered(&self.world.map.tiled);
        self.world.cam_follow.update_follow(
            &mut self.world.cam,
            player_pos,
            Vec2f::new(ra::width_f(), ra::height_f()),
        );

        self.world_render.post_update(&self.world, self.ice.dt);
        self.ui.update(self.ice.dt);
    }

    pub fn pre_render(&mut self) {
        self.ice.pre_render();
    }

    pub fn on_end_frame(&mut self) {
        self.ice.on_end_frame();
    }
}
