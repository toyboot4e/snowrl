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

// TODO: fix spike

use anyhow::{Error, Result};

/// Boilerplate code to run the game with `snow2d::GameRunner`
fn main() -> Result<()> {
    env_logger::init();

    let mut state = snowrl::init()?;
    let pump = state.0.sdl.event_pump().map_err(Error::msg)?;
    let mut fps = snow2d::Fps::default();

    snow2d::run(
        pump,
        &mut state,
        |(ref mut platform, ref mut app), ev| {
            app.event(&ev, platform);
        },
        |(ref mut platform, ref mut app), dt| {
            fps.update(dt);
            log::trace!("FPS: {:.1}, {:.1}", fps.avg(), fps.spike());

            app.update(dt, platform);
            app.render(dt, platform);
        },
    );

    Ok(())
}
