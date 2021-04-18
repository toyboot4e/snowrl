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

    fn debug_update(&mut self, dt: Duration) {
        self.imgui.update_delta_time(dt);
    }

    fn debug_render(&mut self, platform: &mut Platform) {
        let mut ui = self.imgui.begin_frame(&platform.win);
        crate::debug::debug_render(&mut self.data, &mut ui);
        ui.end_frame(&mut platform.win, &mut ()).unwrap();
    }
}

mod sdl2_impl {
    //! Rust-SDL2 support

    use std::time::Duration;

    use sdl2::event::Event;

    use super::Platform;
    use crate::GrueRl;

    /// Lifecycle methods
    impl GrueRl {
        pub fn event(&mut self, ev: &Event, platform: &Platform) {
            self.data.ice.event(ev);
            self.imgui.handle_event(&platform.win, ev);
        }

        pub fn update(&mut self, dt: std::time::Duration, _platform: &mut Platform) {
            self.pre_update(dt);
            self.debug_update(dt);
            self.fsm.update(&mut self.data, &mut self.ctrl);
            self.post_update(dt);
        }

        pub fn pre_render(&mut self, _dt: Duration, platform: &mut Platform) {
            let size = platform.win.size();

            self.data.ice.pre_render(snow2d::gfx::WindowState {
                w: size.0,
                h: size.1,
                // FIXME: never hard code this value
                // dpi_scale: [2.0, 2.0],
                dpi_scale: [1.0, 1.0],
            });
        }

        pub fn post_render(&mut self, dt: Duration, platform: &mut Platform) {
            self.debug_render(platform);
            self.data.ice.post_render(dt);
        }

        pub fn on_end_frame(&mut self) {
            self.data.ice.on_end_frame();
        }
    }
}
