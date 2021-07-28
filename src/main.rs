/*!
Run the game

# TODOs
* fix the slow down on key press

* remove debug/error log on release build?
* inspect Pool/Anim and see if there's garbage
* fix dpi scaling (WindowState, GrueRl::pre_render, begin_default_pass)
* allow resizing window

* put game logic in one place and see app lifecycle from code
  * list of tyobj
*/

use anyhow::{Error, Result};
use sdl2::event::Event;

/// Boilerplate code to run the game with `snow2d::GameRunner`
fn main() -> Result<()> {
    env_logger::init();

    let (mut platform, mut app) = snowrl::init()?;

    let mut pump = platform.sdl.event_pump().map_err(Error::msg)?;
    let mut runner = snow2d::GameRunner::new();
    let mut fps = snow2d::Fps::new();

    // FIXME: freeze on key press
    'game_loop: loop {
        for ev in pump.poll_iter() {
            if matches!(ev, Event::Quit { .. }) {
                break 'game_loop;
            }

            runner.event(&ev);
            app.event(&ev, &mut platform);
        }

        let tick = runner.update();

        if !tick {
            // not focused
            std::thread::sleep(std::time::Duration::from_secs_f32(0.2));
            continue;
        }

        if let Some(dt) = runner.timestep() {
            // focused & update

            // FIXME: consider when not focused)
            fps.update();
            log::trace!("FPS: {:.1}, {:.1}", fps.avg(), fps.spike());

            app.update(dt, &mut platform);
            app.render(dt, &mut platform);
        } else {
            // focusd & iding
            if let Some(dt) = runner.sleep_duration() {
                std::thread::sleep(dt);
            } else {
                eprintln!("ASSSSSSSSSSSSSSSSSSSSSSSS??");
            }
        }
    }

    Ok(())
}
