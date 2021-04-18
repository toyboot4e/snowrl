/*!
Application/platform support
*/

use std::time::{Duration, Instant};

use anyhow::{Error, Result};
use sdl2::event::{Event, WindowEvent};

/// Utility for initializing the game window
pub type Init = rokol::glue::sdl::Init;

#[cfg(feature = "sdl2")]
/// Handles of platform-dependenct RAII objects
pub type Platform = rokol::glue::sdl::WindowHandle;

/// Platform-independent application lifecycle
pub trait Lifecycle {
    type Event;
    fn event(&mut self, ev: Self::Event);
    fn update(&mut self, dt: Duration, platform: &mut Platform);
    fn render(&mut self, dt: Duration, platform: &mut Platform);
}

/// Run an application that implements [`Lifecycle`]
pub fn run<A>(mut platform: Platform, mut app: A) -> Result<()>
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

    use super::Platform;
    use crate::GrueRl;

    /// Lifecycle methods
    impl GrueRl {
        pub fn event(&mut self, ev: &Event) {
            self.data.ice.event(ev);
        }

        pub fn update(&mut self, dt: std::time::Duration, _platform: &mut Platform) {
            self.pre_update(dt);
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

        pub fn post_render(&mut self, dt: Duration) {
            self.data.ice.post_render(dt);
        }

        pub fn on_end_frame(&mut self) {
            self.data.ice.on_end_frame();
        }
    }
}
