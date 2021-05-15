/*!
Run the game

# TODOs
* remove debug/error log on release build?
* inspect Pool/Anim and see if there's garbage
* fix dpi scaling (WindowState, GrueRl::pre_render, begin_default_pass)
* allow resizing window

* put game logic in one place and see app lifecycle from code
  * list of tyobj
*/

use std::time::Duration;

use anyhow::{Error, Result};
use sdl2::event::Event;

fn main() -> Result<()> {
    env_logger::init();

    let (mut platform, mut app) = snowrl::init()?;

    let mut pump = platform.sdl.event_pump().map_err(Error::msg)?;
    let mut runner = snow2d::GameRunner::new();

    'running: loop {
        for ev in pump.poll_iter() {
            match ev {
                Event::Quit { .. } => break 'running,
                _ => {}
            }

            runner.event(&ev);
            app.event(&ev, &mut platform);
        }

        if runner.update() {
            app.update(runner.dt(), &mut platform);
            app.render(runner.dt(), &mut platform);
        }

        std::thread::sleep(Duration::from_micros(100));
    }

    Ok(())
}
