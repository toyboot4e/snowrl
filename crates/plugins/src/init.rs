mod init_res;

use snow2d::{
    gfx::{Snow2d, WindowState},
    ui::Ui,
};

use grue2d::{
    app::Platform,
    fsm::Fsm,
    game::{
        cfg::*,
        data::res::{Resources, VInput},
        Control, Data,
    },
    markup::KbdIcons,
};

use crate::{prelude::*, states};

pub fn gen_app(w: u32, h: u32) -> Result<Platform> {
    let init = grue2d::app::Init {
        title: "SnowRL".to_string(),
        w,
        h,
        use_high_dpi: false,
        ..Default::default()
    };

    init.init(|w| {
        w.position_centered();
        // w.allow_hidhdpi();
    })
    .map_err(Error::msg)
}

/// Creates [`GrueRl`] on startup
pub fn new_game(w: u32, h: u32) -> Result<(Data, Control, Fsm)> {
    // create our game context
    let mut data = {
        let mut ice = Ice::new(unsafe {
            Snow2d::new(WindowState {
                w,
                h,
                // TODO: remove the magic scaling number
                dpi_scale: [2.0, 2.0],
            })
        });

        init_res::init_assets(&mut ice).unwrap();

        let mut ui = Ui::new();
        let world = init_res::init_world([w, h], &mut ice, &mut ui).unwrap();

        let fonts = init_res::load_fonts(&mut ice);

        let kbd_icons = {
            let kbd_icons_tex = ice
                .assets
                .load_sync_preserve(grue2d::paths::img::kbd::KBD_2)?;

            KbdIcons::new(
                kbd_icons_tex,
                &ice.assets.resolve(grue2d::paths::img::kbd::KBD_2_PACK),
            )?
        };

        Data {
            ice,
            world,
            res: Resources {
                fonts,
                kbd_icons,
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

    Ok((data, ctrl, fsm))
}
