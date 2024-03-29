/*!
Framework for SnowRL

Based on [`rlbox`] (roguelike toolbox) and [`snow2d`] (2D framework)
*/

#![feature(
    generators,
    generator_trait,
    array_map,
    drain_filter,
    const_raw_ptr_deref
)]

pub extern crate rlbox;

pub mod app;
pub mod fsm;
pub mod game;
pub mod markup;
pub mod paths;

#[cfg(debug_assertions)]
pub mod debug;

use anyhow::*;
use rokol::gfx as rg;
use snow2d::{gfx::geom2d::Vec2f, ui::CoordSystem};
use std::time::Duration;

use sdl2::event::Event;

use crate::{
    app::Platform,
    fsm::*,
    game::{agents::WorldRenderer, data::res::UiLayer, Agents, Control, Data},
};

/// Pass action that clears the screen with cornflower blue
pub const PA_BLUE: rg::PassAction =
    rg::PassAction::clear_const([100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 250.0 / 255.0]);

/// Logic that operates on data ([`GrueRl`])
pub trait Plugin: std::fmt::Debug {
    /// Create components of [`GrueRl`]
    fn init_game(&mut self) -> Result<(Platform, (Data, Control, Fsm))>;
    fn on_load(&mut self, grue: &mut GrueRl, platform: &mut Platform);
    fn event(&mut self, grue: &mut GrueRl, ev: &Event, platform: &mut Platform);
    fn update(&mut self, grue: &mut GrueRl, dt: Duration, platform: &mut Platform);
    fn render(&mut self, grue: &mut GrueRl, dt: Duration, platform: &mut Platform);
}

/// All of the game data: [`Data`], [`Control`], [`Agents`] and [`Fsm`]
///
/// [`Fsm`] controls the game. [`Agents`] work on the game state. [`Data`] is a set of passive data.
#[derive(Debug)]
pub struct GrueRl {
    /// Passive data
    pub data: Data,
    /// States to control the game
    pub ctrl: Control,
    /// Objects with exclusive data
    pub agents: Agents,
    /// Controls the game
    pub fsm: Fsm,
    #[cfg(debug_assertions)]
    /// (Debug-only) ImGUI
    pub imgui: debug::Backend,
    /// (Debug-only) Debug state
    #[cfg(debug_assertions)]
    pub debug_state: debug::DebugState,
}

impl GrueRl {
    pub fn new(platform: &Platform, data: Data, ctrl: Control, fsm: Fsm) -> Result<Self> {
        let screen_size = [platform.win.size().0, platform.win.size().1];
        let agents = Agents::new(screen_size, &data.ice.snow.clock);

        #[cfg(debug_assertions)]
        let imgui = debug::create_backend(platform)?;

        Ok(Self {
            data,
            ctrl,
            agents,
            fsm,
            #[cfg(debug_assertions)]
            imgui,
            #[cfg(debug_assertions)]
            debug_state: Default::default(),
        })
    }
}

/// Lifecycle components
impl GrueRl {
    /// Called before updating the FSM (game state). Ticks input/graphics times
    #[inline(always)]
    fn pre_update(&mut self, dt: Duration) {
        let data = &mut self.data;
        data.ice.pre_update(dt);
        data.world.update(&mut data.ice);
        data.res.vi.update(&data.ice.input, dt);
    }

    /// Called after updating the FSM (game state). Updates buffers and ticks UI state
    #[inline(always)]
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
        data.res.dir_anims.update(dt, &mut data.res.ui);
    }

    #[cfg(debug_assertions)]
    #[inline(always)]
    fn debug_update(&mut self, dt: Duration) {
        self.imgui.update_delta_time(dt);
    }

    #[cfg(debug_assertions)]
    #[inline(always)]
    fn debug_render(&mut self, platform: &mut Platform) {
        let mut ui = self.imgui.begin_frame(&platform.win);
        self.debug_state
            .debug_render(&mut self.data, &mut self.ctrl, &mut ui);
        ui.end_frame(&mut platform.win, &mut ()).unwrap();
    }
}

mod sdl2_impl {
    //! Rust-SDL2 lifecycle methods

    use std::time::Duration;

    use sdl2::event::Event;

    use super::Platform;
    use crate::{game::data::res::UiLayer, DrawStage, GrueRl};

    /// Lifecycle methods
    impl GrueRl {
        #[inline(always)]
        pub fn event(&mut self, ev: &Event, platform: &Platform) {
            self.data.ice.event(ev);
            #[cfg(debug_assertions)]
            self.imgui.handle_event(&platform.win, ev);
        }

        #[inline(always)]
        pub fn update(&mut self, dt: std::time::Duration, _platform: &mut Platform) {
            self.pre_update(dt);
            #[cfg(debug_assertions)]
            self.debug_update(dt);
            self.fsm.update(&mut self.data, &mut self.ctrl);
            self.post_update(dt);
        }

        /// Render the game in default order
        #[inline(always)]
        pub fn render_default(&mut self) {
            pub const DEFAULT_RENDER_SCHEDULE: &'static [DrawStage] = &[
                DrawStage::MapDown,
                DrawStage::UiLayer(UiLayer::Actors),
                DrawStage::UiLayer(UiLayer::OnActors),
                DrawStage::MapUp,
                DrawStage::Shadow,
                DrawStage::UiLayer(UiLayer::OnShadow),
                DrawStage::Snow,
                DrawStage::UiLayer(UiLayer::Screen),
            ];

            crate::run_scheduled_render(DEFAULT_RENDER_SCHEDULE, self);
        }

        #[inline(always)]
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

        #[inline(always)]
        pub fn post_render(&mut self, dt: Duration, platform: &mut Platform) {
            #[cfg(debug_assertions)]
            self.debug_render(platform);
            self.data.ice.post_render(dt);
        }

        #[inline(always)]
        pub fn on_end_frame(&mut self) {
            self.data.ice.on_end_frame();
        }
    }
}

/// Component of rendering schedule
#[derive(Debug, Clone, Copy)]
pub enum DrawStage {
    UiLayer(crate::game::data::res::UiLayer),
    /// Down parts of the map
    MapDown,
    /// Up parts of the map
    MapUp,
    Shadow,
    Snow,
    /// Clear screen with cornflower blue
    ClearScreen,
}

#[inline(always)]
pub fn run_scheduled_render(schedule: &[DrawStage], grue: &mut GrueRl) {
    for stage in schedule {
        stage.draw(grue);
    }
}

impl DrawStage {
    #[inline(always)]
    pub fn draw(self, grue: &mut GrueRl) {
        let (data, agents) = (&mut grue.data, &mut grue.agents);
        let cam_mat = data.world.cam.to_mat4();

        let (ice, res, world, cfg) = (&mut data.ice, &mut data.res, &mut data.world, &data.cfg);
        let dt = ice.dt();

        match self {
            DrawStage::UiLayer(ui_layer) => {
                if ui_layer == UiLayer::Actors {
                    // FIXME: we're assuming `OnActors` is drawn actor `Actors`
                    agents
                        .world_render
                        .setup_actor_nodes(world, &mut res.ui, dt);
                }

                let mut screen = ice
                    .snow
                    .screen()
                    .transform(match ui_layer.to_layer().coord {
                        CoordSystem::Screen => None,
                        CoordSystem::World => Some(cam_mat),
                    })
                    .build();

                res.ui.render_range(ui_layer.to_draw_range(), &mut screen);
            }
            DrawStage::MapDown => {
                let mut screen = ice
                    .snow
                    .screen()
                    .pa(Some(&rg::PassAction::LOAD))
                    .transform(Some(cam_mat))
                    .build();
                WorldRenderer::render_map(&mut screen, world, 0..100);
            }
            DrawStage::MapUp => {
                let mut screen = ice
                    .snow
                    .screen()
                    .pa(Some(&PA_BLUE))
                    .transform(Some(cam_mat))
                    .build();
                WorldRenderer::render_map(&mut screen, world, 100..);
            }
            DrawStage::Shadow => {
                agents
                    .world_render
                    .render_shadow(&mut ice.snow, world, &cfg.shadow_cfg);
            }
            DrawStage::Snow => {
                agents
                    .world_render
                    .render_snow(&ice.snow.window, &ice.snow.clock, &cfg.snow_cfg);
            }
            DrawStage::ClearScreen => {
                // TODO: is this inefficient
                let _screen = ice.snow.screen().pa(Some(&PA_BLUE)).build();
            }
        }
    }
}
