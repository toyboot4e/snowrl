/*!
SnowRL framework based on [`rlbox`] (roguelike toolbox) and [`snow2d`] (2D framework)
*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub extern crate rlbox;

pub mod agents;
pub mod ctrl;
pub mod data;
pub mod fsm;

pub mod platform;

use std::time::Duration;

use snow2d::gfx::geom2d::Vec2f;

use crate::{agents::Agents, ctrl::Control, data::Data, fsm::*};

pub extern crate hot_crate;

/// TODO: Plugin-based game content?
pub trait Plugin: std::fmt::Debug {}

/// All of the game data: [`Data`], [`Control`], [`Agents`] and [`Fsm`]
///
/// [`Fsm`] controls the game. [`Agents`] work on the game state. [`Data`] is a set of passive data.
#[derive(Debug)]
pub struct GrueRl {
    /// Passive data
    pub data: Data,
    /// States to control the game
    pub ctrl: Control,
    /// Objects for the state machine
    pub agents: Agents,
    /// Controls the game
    pub fsm: Fsm,
}

impl GrueRl {
    pub fn new(screen_size: [u32; 2], data: Data, fsm: Fsm) -> Self {
        Self {
            data,
            ctrl: Control::new(),
            agents: Agents::new(screen_size),
            fsm,
        }
    }
}

/// Lifecycle components
impl GrueRl {
    /// Ticks input/graphics times
    //
    /// Called before updating the FSM (game state).
    fn pre_update(&mut self, dt: Duration) {
        let data = &mut self.data;
        data.ice.pre_update(dt);
        data.world.update(&mut data.ice);
        data.res.vi.update(&data.ice.input, dt);
    }

    /// Updates buffers and ticks UI state
    //
    /// Called after updating the FSM (game state).
    fn post_update(&mut self) {
        let (data, agents) = (&mut self.data, &mut self.agents);
        let dt = data.ice.dt();

        // shadow
        // TODO: don't hard code player detection
        let player = &data.world.entities.get_by_slot(0).unwrap().1;
        data.world
            .shadow
            .post_update(dt, &data.world.map.rlmap, player.pos);

        // camera
        let player_pos = player.img.pos_world_centered(&data.world.map.tiled);
        data.world.cam_follow.update_follow(
            &mut data.world.cam,
            player_pos,
            Vec2f::from(data.ice.snow.window.size_f32()),
        );

        agents.world_render.post_update(&data.world, dt);
        data.res.ui.update(dt);
    }
}
