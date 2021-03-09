/*!
Framework of SnowRL based on [`rlbox`]
*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub extern crate rlbox;

pub mod agents;
pub mod data;
pub mod fsm;

use {
    rokol::{
        app::{self as ra, glue::Rokol, RApp},
        gfx as rg,
    },
    snow2d::gfx::geom2d::Vec2f,
};

use crate::{agents::Agents, data::Data, fsm::*};

pub extern crate hot_crate;
pub trait Plugin: std::fmt::Debug {}

/// Runs [`RApp`], which provides 60 FPS fixed-timestep game loop
pub fn run<App: RApp, AppConstructor: FnOnce(Rokol) -> App>(
    rokol: Rokol,
    constructor: AppConstructor,
) -> ra::glue::Result {
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
        rg::setup(&mut rokol::app::glue::app_desc());
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

#[derive(Debug)]
pub struct GrueRl {
    pub data: Data,
    pub agents: Agents,
    pub fsm: Fsm,
}

impl GrueRl {
    pub fn new(data: Data, fsm: Fsm) -> Self {
        Self {
            data,
            agents: Agents::new(),
            fsm,
        }
    }
}

/// Lifecycle methods to be used by a driver of `Fsm`
impl GrueRl {
    pub fn event(&mut self, ev: &ra::Event) {
        self.data.ice.event(ev);
    }

    /// Ticks input/graphics times
    //
    /// Called before updating the FSM (game state).
    fn pre_update(&mut self) {
        let data = &mut self.data;
        data.ice.pre_update();
        data.world.update(&mut data.ice);
        data.res.vi.update(&data.ice.input, data.ice.dt);
    }

    /// Updates buffers and ticks UI state
    //
    /// Called after updating the FSM (game state).
    fn post_update(&mut self) {
        let (data, agents) = (&mut self.data, &mut self.agents);

        // shadow
        // TODO: don't hard code player detection
        let player = &data.world.entities.get_by_slot(0).unwrap().1;
        data.world
            .shadow
            .post_update(data.ice.dt, &data.world.map.rlmap, player.pos);

        // camera
        let player_pos = player.img.pos_world_centered(&data.world.map.tiled);
        data.world.cam_follow.update_follow(
            &mut data.world.cam,
            player_pos,
            Vec2f::new(ra::width_f(), ra::height_f()),
        );

        agents.world_render.post_update(&data.world, data.ice.dt);
        data.res.ui.update(data.ice.dt);
    }

    pub fn update(&mut self) {
        self.pre_update();
        self.fsm.update(&mut self.data);
        self.post_update();
    }

    pub fn pre_render(&mut self) {
        self.data.ice.pre_render();
    }

    pub fn on_end_frame(&mut self) {
        self.data.ice.on_end_frame();
    }
}
