/*!
Framework for SnowRL

Based on [`rlbox`] (roguelike toolbox) and [`snow2d`] (2D framework)
*/

#![feature(generators, generator_trait, array_map)]

pub extern crate hot_crate;
pub extern crate rlbox;

pub mod app;
pub mod fsm;
pub mod game;

#[cfg(debug_assertions)]
pub mod debug;

use std::time::Duration;

use anyhow::*;

use snow2d::gfx::geom2d::Vec2f;

use crate::{
    app::Platform,
    fsm::*,
    game::{Agents, Control, Data},
};

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
    #[cfg(debug_assertions)]
    /// (Debug-only) ImGUI
    pub imgui: debug::Backend,
}

impl GrueRl {
    pub fn new(platform: &Platform, data: Data, fsm: Fsm, ctrl: Control) -> Result<Self> {
        let screen_size = [platform.win.size().0, platform.win.size().1];
        let agents = Agents::new(screen_size, &data.ice.snow.clock);
        let imgui = debug::create_backend(platform)?;

        Ok(Self {
            data,
            ctrl,
            agents,
            fsm,
            imgui,
        })
    }
}

/// Lifecycle components
impl GrueRl {
    /// Called before updating the FSM (game state). Ticks input/graphics times
    fn pre_update(&mut self, dt: Duration) {
        let data = &mut self.data;
        data.ice.pre_update(dt);
        data.world.update(&mut data.ice);
        data.res.vi.update(&data.ice.input, dt);
    }

    /// Called after updating the FSM (game state). Updates buffers and ticks UI state
    fn post_update(&mut self, dt: Duration) {
        let (data, agents) = (&mut self.data, &mut self.agents);

        // shadow
        // FIXME: don't hard code player detection
        const PLAYER_SLOT: u32 = 0;
        let player = &data.world.entities.get_by_slot(PLAYER_SLOT).unwrap().1;
        data.world
            .shadow
            .post_update(dt, &data.world.map.rlmap, player.pos);

        // camera
        let player_pos = player.view.pos_world_centered(&data.world.map.tiled);
        data.world.cam_follow.update_follow(
            &mut data.world.cam,
            player_pos,
            Vec2f::from(data.ice.snow.window.size_f32()),
        );

        agents.world_render.post_update(&data.world, dt);
        data.res.ui.update(dt);
    }
}
