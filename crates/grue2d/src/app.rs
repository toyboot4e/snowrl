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
    fn event(&mut self, ev: Self::Event, platform: &mut Platform);
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
                _ => {}
            }

            app.event(ev, &mut platform);
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
