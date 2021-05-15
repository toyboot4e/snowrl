/*!
Boostrap `SnowRl`

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

use snow2d::{gfx::WindowState, ui::Ui};

use grue2d::{
    app::Platform,
    game::{
        cfg::*,
        data::res::{Resources, VInput},
        Control,
    },
};

use snowrl::{init, prelude::*, states, SnowRl};

fn main() -> Result<()> {
    env_logger::init();

    let (mut platform, mut app) = self::gen_app()?;

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

        // std::thread::sleep(Duration::from_micros(100));
        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

fn gen_app() -> Result<(Platform, SnowRl)> {
    let init = grue2d::app::Init {
        title: "SnowRL".to_string(),
        // FIXME: magic value
        w: 1280,
        h: 720,
        use_high_dpi: false,
        ..Default::default()
    };

    let mut platform = init
        .init(|w| {
            w.position_centered();
            // w.allow_hidhdpi();
        })
        .map_err(Error::msg)?;

    let app = SnowRl::new(self::new_game(&init, &mut platform).unwrap());

    Ok((platform, app))
}

fn new_game(init: &grue2d::app::Init, platform: &Platform) -> Result<GrueRl> {
    // create our game context
    let mut data = {
        let mut ice = Ice::new(unsafe {
            Snow2d::new(WindowState {
                w: init.w,
                h: init.h,
                // TODO: remove magic scaling number
                dpi_scale: [2.0, 2.0],
            })
        });
        init::init_assets(&mut ice).unwrap();

        let mut ui = Ui::new();
        let screen_size = [init.w, init.h];
        let world = init::init_world(screen_size, &mut ice, &mut ui).unwrap();
        let fonts = init::load_fonts(&mut ice);

        Data {
            ice,
            world,
            res: Resources {
                fonts,
                vi: VInput::new(),
                ui,
                dir_anims: Default::default(),
            },
            cfg: GameConfig {
                vol: 1.0,
                shadow_cfg: ShadowConfig::Blur,
                snow_cfg: SnowConfig::Blizzard,
            },
        }
    };

    let mut ctrl = Control::new();

    // create our control
    let fsm = {
        let mut fsm = grue2d::fsm::Fsm::default();

        fsm.insert_default::<states::Roguelike>();
        fsm.insert_default::<states::Animation>();

        fsm.insert(states::Title::new(&mut data.ice, &mut data.res.ui));

        fsm.insert(states::PlayScript::new(&mut data.ice.assets));

        fsm.push::<states::Roguelike>(&mut data, &mut ctrl);
        fsm.push::<states::Title>(&mut data, &mut ctrl);

        fsm
    };

    GrueRl::new(platform, data, fsm, ctrl)
}
