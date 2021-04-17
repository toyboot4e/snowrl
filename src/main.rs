/*!
SnowRL

# TODOs
* remove debug/error log on release build?
* inspect Pool/Anim and see if there's garbage
* fix dpi scaling (WindowState, GrueRl::pre_render, begin_default_pass)
* allow resizing window
*/

use anyhow::{Error, Result};

use snow2d::gfx::WindowState;

use grue2d::{
    app::Platform,
    game::{
        data::res::{Resources, Ui, VInput},
        Control,
    },
};

use snowrl::{init, prelude::*, states, SnowRl};

fn main() -> Result<()> {
    env_logger::init();

    let init = grue2d::app::Init {
        title: "SnowRL".to_string(),
        // FIXME: magic value
        w: 1280,
        h: 720,
        use_high_dpi: false,
        ..Default::default()
    };

    let platform = init
        .init(|w| {
            w.position_centered();
            // w.allow_hidhdpi();
        })
        .map_err(Error::msg)?;

    let mut game = self::new_game(&init, &platform)?;
    game.data.ice.audio.set_global_volume(0.0);
    let app = SnowRl::new(game);

    grue2d::app::run(platform, app)
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
