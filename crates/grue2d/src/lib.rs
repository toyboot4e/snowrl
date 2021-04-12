/*!
Framework for SnowRL

Based on [`rlbox`] (roguelike toolbox) and [`snow2d`] (2D framework)
*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub extern crate hot_crate;
pub extern crate rlbox;

pub mod agents;
pub mod ctrl;
pub mod data;
pub mod fsm;

use std::time::{Duration, Instant};

use ::sdl2::event::Event;
use anyhow::{Error, Result};

use snow2d::gfx::geom2d::Vec2f;

use crate::{agents::Agents, ctrl::Control, data::Data, fsm::*};

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

/// Utility for initializing the game window
pub type Init = rokol::glue::sdl::Init;

/// Handles of platform-dependenct RAII objects
pub type PlatformLifetime = rokol::glue::sdl::WindowHandle;

/// Platform-independent application lifecycle
pub trait Lifecycle {
    type Event;
    fn event(&mut self, ev: Self::Event);
    fn update(&mut self, dt: Duration, platform: &mut PlatformLifetime);
    fn render(&mut self, dt: Duration, platform: &mut PlatformLifetime);
}

/// Run an application that implements [`Lifecycle`]
pub fn run<A>(mut platform: PlatformLifetime, mut app: A) -> Result<()>
where
    A: Lifecycle<Event = ::sdl2::event::Event>,
{
    let mut pump = platform.sdl.event_pump().map_err(Error::msg)?;

    // FIXME: use accurate FPS
    let target_dt = Duration::from_nanos(1_000_000_000 / 60);
    let mut now = Instant::now();
    let mut accum = Duration::default();

    'running: loop {
        for ev in pump.poll_iter() {
            match ev {
                Event::Quit { .. } => break 'running,
                _ => {
                    app.event(ev);
                }
            }
        }

        let new_now = Instant::now();
        accum += new_now - now;
        now = new_now;

        app.update(target_dt, &mut platform);
        app.render(target_dt, &mut platform);

        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

mod sdl2_impl {
    //! Rust-SDL2 support

    use sdl2::event::Event;

    use super::PlatformLifetime;
    use crate::GrueRl;

    /// Lifecycle methods
    impl GrueRl {
        pub fn event(&mut self, ev: &Event) {
            self.data.ice.event(ev);
        }

        pub fn update(&mut self, dt: std::time::Duration, _platform: &mut PlatformLifetime) {
            self.pre_update(dt);
            self.fsm.update(&mut self.data, &mut self.ctrl);
            self.post_update(dt);
        }

        pub fn pre_render(&mut self, _dt: std::time::Duration, platform: &mut PlatformLifetime) {
            let size = platform.win.size();

            self.data.ice.pre_render(snow2d::gfx::WindowState {
                w: size.0,
                h: size.1,
                // FIXME: never hard code this value
                // dpi_scale: [2.0, 2.0],
                dpi_scale: [1.0, 1.0],
            });
        }

        pub fn on_end_frame(&mut self) {
            self.data.ice.on_end_frame();
        }
    }
}

// mod rokol_impl {
//     //! `rokol:app` support
//
//     use rokol::app::{self as ra, glue::Rokol, RApp};
//
//     /// Runs [`RApp`], which provides 60 FPS fixed-timestep game loop
//     pub fn run<App: RApp, AppConstructor: FnOnce(Rokol) -> App>(
//         rokol: Rokol,
//         constructor: AppConstructor,
//     ) -> ra::glue::Result {
//         rokol.run(&mut Runner {
//             init_rokol: Some(rokol.clone()),
//             init: Some(constructor),
//             x: None,
//         })
//     }
//
//     /// Creates [`RApp`] _after_ creating `rokol::gfx` contexts
//     struct Runner<T: RApp, F: FnOnce(Rokol) -> T> {
//         init_rokol: Option<Rokol>,
//         /// Use `Option` for lazy initialization
//         init: Option<F>,
//         /// Use `Option` for lazy initialization
//         x: Option<T>,
//     }
//
//     impl<T: RApp, F: FnOnce(Rokol) -> T> rokol::app::RApp for Runner<T, F> {
//         fn init(&mut self) {
//             rg::setup(&mut rokol::app::glue::app_desc());
//             let f = self.init.take().unwrap();
//             self.x = Some(f(self.init_rokol.take().unwrap()));
//         }
//
//         fn event(&mut self, ev: &ra::Event) {
//             if let Some(x) = self.x.as_mut() {
//                 x.event(ev);
//             }
//         }
//
//         fn frame(&mut self) {
//             if let Some(x) = self.x.as_mut() {
//                 x.frame();
//             }
//         }
//     }
//
//     /// Lifecycle methods to be used by a driver of `Fsm`
//     impl GrueRl {
//         pub fn event(&mut self, ev: &Event) {
//             self.data.ice.event(ev);
//         }
//
//         pub fn update(&mut self) {
//             self.pre_update();
//             self.fsm.update(&mut self.data);
//             self.post_update();
//         }
//
//         pub fn pre_render(&mut self) {
//             self.data.ice.pre_render(snow2d::input::WindowState {
//                 w: rokol::app: w(),
//                 h: rokol::app: h(),
//             });
//         }
//
//         pub fn on_end_frame(&mut self) {
//             self.data.ice.on_end_frame();
//         }
//     }
// }
