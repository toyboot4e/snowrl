/*!
Game loop and lifecycle
*/

pub mod sdl2;

/// Handles of platform-dependenct RAII objects
pub type PlatformLifetime = rokol::glue::sdl::WindowHandle;

pub type Init = rokol::glue::sdl::Init;

use std::time::{Duration, Instant};

use ::sdl2::event::Event;
use anyhow::{Error, Result};

/// Platform-independent application lifecycle
pub trait Lifecycle {
    type Event;
    fn event(&mut self, ev: Self::Event);
    fn update(&mut self, dt: Duration, platform: &mut PlatformLifetime);
    fn render(&mut self, dt: Duration, platform: &mut PlatformLifetime);
}

pub fn run<A>(mut platform: PlatformLifetime, mut app: A) -> Result<()>
where
    A: Lifecycle<Event = ::sdl2::event::Event>,
{
    let mut pump = platform.sdl.event_pump().map_err(Error::msg)?;

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
