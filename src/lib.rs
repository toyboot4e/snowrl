/*!

Snow the roguelike game

*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub use {rlbox, rokol, snow2d};

pub mod state;
pub mod turn;
pub mod utils;
pub mod world;

use rokol::{app as ra, gfx as rg};

use crate::{
    turn::{
        anim::{AnimPlayer, AnimResult, AnimUpdateContext},
        tick::{AnimContext, GameLoop, TickResult},
    },
    world::{World, WorldContext},
};

pub fn run(app: rokol::Rokol) -> rokol::Result {
    app.run(&mut SnowRl::new())
}

pub struct SnowRl {
    /// Use `Option` for lazy initialization
    x: Option<SnowRlImpl>,
}

impl SnowRl {
    pub fn new() -> Self {
        Self { x: None }
    }
}

/// Delay the initialization
impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());
        self.x = Some(SnowRlImpl::new());
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

#[derive(Debug)]
struct SnowRlImpl {
    gl: state::Global,
    fsm: state::Fsm,
}

impl SnowRlImpl {
    pub fn new() -> Self {
        let file = snow2d::asset::path("map/tmx/title.tmx");

        let mut wcx = WorldContext::new();
        let world = World::from_tiled_file(&mut wcx, &file).unwrap();

        let mut gl = state::Global {
            world,
            wcx,
            anims: AnimPlayer::new(),
        };

        let fsm = {
            let mut fsm = state::Fsm::new();
            fsm.insert_default::<state::Roguelike>();
            fsm.insert_default::<state::Animation>();
            fsm.push::<state::Roguelike>(&mut gl);
            fsm
        };

        log::trace!("init FSM");

        Self { gl, fsm }
    }
}

impl rokol::app::RApp for SnowRlImpl {
    fn event(&mut self, ev: &ra::Event) {
        self.gl.wcx.event(ev);
        self.gl.world.event(&mut self.gl.wcx, ev);
    }

    fn frame(&mut self) {
        // update the internal game state
        self.gl.wcx.update();
        self.fsm.update(&mut self.gl);

        // update view states
        self.gl.world.update_images(&mut self.gl.wcx);
        self.gl.world.shadow.update(self.gl.wcx.dt);

        // finally render them all
        self.gl.wcx.render();
        self.gl.world.render(&mut self.gl.wcx);

        self.gl.wcx.on_end_frame();
        self.gl.world.on_end_frame(&mut self.gl.wcx);

        rg::commit();
    }
}

pub mod consts {
    //! Magic values (should be removed)

    /// FPS of character graphics animation
    pub const ACTOR_FPS: f32 = 4.0;

    /// Filed of view radius
    pub const FOV_R: u32 = 5;

    /// Walk duration in seconds
    pub const WALK_TIME: f32 = 8.0 / 60.0;

    /// Half frame in seconds (fixed timestep with 60 FPS)
    pub const HALF_FRAME: f32 = 1.0 / 120.0;
}
