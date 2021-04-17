/*!
Application/platform support
*/

use std::time::{Duration, Instant};

use anyhow::{Error, Result};
use sdl2::event::{Event, WindowEvent};

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

    // new, previous
    let mut focus = [false, false];

    'running: loop {
        for ev in pump.poll_iter() {
            match ev {
                Event::Quit { .. } => break 'running,
                Event::Window {
                    // main `window_id` is `1`
                    window_id,
                    win_event,
                    ..
                } => match win_event {
                    // keyborad focus
                    WindowEvent::FocusLost => {
                        log::trace!("focus lost: {:?}", window_id);
                        focus[1] = false;
                    }
                    WindowEvent::FocusGained => {
                        log::trace!("gain: {:?}", window_id);
                        focus[1] = true;
                    }
                    // window focus (take only)
                    WindowEvent::TakeFocus => {
                        log::trace!("take: {:?}", window_id);
                    }
                    // window status
                    WindowEvent::Shown => {
                        log::trace!("shown: {:?}", window_id);
                    }
                    WindowEvent::Hidden => {
                        log::trace!("hidden: {:?}", window_id);
                    }
                    WindowEvent::Exposed => {
                        log::trace!("exposed: {:?}", window_id);
                    }
                    WindowEvent::Close => {
                        log::trace!("closed: {:?}", window_id);
                    }
                    WindowEvent::HitTest => {
                        log::trace!("hit-test: {:?}", window_id);
                    }
                    // window placement
                    WindowEvent::Moved(x, y) => {
                        log::trace!("moved: {:?} ({:?}, {:?})", window_id, x, y);
                    }
                    WindowEvent::Resized(w, h) => {
                        log::trace!("resized: {:?} ({:?}, {:?})", window_id, w, h);
                    }
                    WindowEvent::SizeChanged(w, h) => {
                        log::trace!("size changed: {:?} ({:?}, {:?})", window_id, w, h);
                    }
                    WindowEvent::Minimized => {
                        log::trace!("minimized: {:?}", window_id);
                    }
                    WindowEvent::Maximized => {
                        log::trace!("maximized: {:?}", window_id);
                    }
                    WindowEvent::Restored => {
                        log::trace!("restored: {:?}", window_id);
                    }
                    // mouse cursor (enter/leave window area)
                    WindowEvent::Enter => {
                        //
                    }
                    WindowEvent::Leave => {
                        //
                    }
                    _ => {}
                },
                _ => {
                    app.event(ev);
                }
            }
        }

        match (focus[0], focus[1]) {
            (false, true) => {
                // gain focus
                accum = Duration::default();
                now = Instant::now();
            }
            (true, false) => {
                // lose focus
                accum = Duration::default();
                now = Instant::now();
            }
            (true, true) => {
                // been focused
                let new_now = Instant::now();
                accum += new_now - now;
                now = new_now;

                // update the game ONLY WHILE FOCUSED
                app.update(target_dt, &mut platform);
                app.render(target_dt, &mut platform);
            }
            (false, false) => {
                // been unfocused: stop the game
            }
        }
        focus[0] = focus[1];

        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

mod sdl2_impl {
    //! Rust-SDL2 support

    use std::time::Duration;

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

        pub fn pre_render(&mut self, _dt: Duration, platform: &mut PlatformLifetime) {
            let size = platform.win.size();

            self.data.ice.pre_render(snow2d::gfx::WindowState {
                w: size.0,
                h: size.1,
                // FIXME: never hard code this value
                // dpi_scale: [2.0, 2.0],
                dpi_scale: [1.0, 1.0],
            });
        }

        pub fn post_render(&mut self, dt: Duration) {
            self.data.ice.post_render(dt);
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
