/*!
Integrate plugins
*/

#![feature(const_raw_ptr_deref)]

pub extern crate gui;
pub extern crate rokol;
pub extern crate snow2d;

pub mod consts;
pub mod paths;

use std::path::PathBuf;

use anyhow::*;

use snow2d::{
    asset::AssetKey,
    gfx::{geom2d::*, tex::Texture2dDrop, Snow2d, WindowState},
    input::Dir8,
    ui::{self, node, Node, Ui},
    utils::{
        arena::{Arena, Index},
        tyobj::TypeObjectStorageBuilder,
    },
    Ice,
};

use gui::{
    app::Platform,
    fsm::Fsm,
    markup::KbdIcons,
    model::{GameSystem, Model},
    res::{self, Resources, VInput},
    spawn::{ActorSpawn, ActorType},
    view::{
        self,
        actor::ActorImageType,
        anim::DirAnimType,
        camera::{Camera2d, FollowCamera2d, Transform2dParams},
        shadow::Shadow,
    },
    Data, GameConfig, Gui, ShadowConfig, SnowConfig,
};

#[derive(Debug)]
pub struct SnowRl {
    pub data: Data,
    pub fsm: Fsm<Data>,
}

/// Create a window and initialize the game
pub fn init() -> Result<(Platform, SnowRl)> {
    let init = gui::app::Init {
        title: "SnowRL".to_string(),
        w: 1280,
        h: 720,
        use_high_dpi: false,
        ..Default::default()
    };

    let platform = {
        init.init(|w| {
            w.position_centered();
            // w.allow_hidhdpi();
        })
        .map_err(Error::msg)
    }?;

    // ****************************************
    // Disable text input, including IME. This is important for constant FPS
    // (At least on my mac for unknown reason)
    // ****************************************
    platform.vid.text_input().stop();

    let ice = self::gen_ice(init.w, init.h)?;
    let data = self::gen_data(init.w, init.h, ice)?;
    let fsm = Fsm::<Data>::default();

    Ok((platform, SnowRl { data, fsm }))
}

#[cfg(feature = "sdl2")]
mod impl_ {
    use super::*;

    use sdl2::event::Event;
    use std::time::Duration;

    impl SnowRl {
        #[inline(always)]
        pub fn event(&mut self, ev: &Event, platform: &mut Platform) {
            //
        }

        #[inline(always)]
        pub fn update(&mut self, dt: Duration, platform: &mut Platform) {
            //
        }

        #[inline(always)]
        pub fn render(&mut self, dt: Duration, platform: &mut Platform) {
            //
        }
    }
}

fn gen_ice(w: u32, h: u32) -> Result<Ice> {
    let mut ice = {
        let snow = unsafe {
            Snow2d::new(WindowState {
                w,
                h,
                // TODO: remove the magic scaling number
                dpi_scale: [2.0, 2.0],
            })
        };

        // FIXME: Consider release build
        let proj_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let asset_root = PathBuf::from(proj_root).join("assets");

        Ice::new(snow, asset_root)
    };

    init_assets(&mut ice)?;

    return Ok(ice);

    fn init_assets(ice: &mut Ice) -> anyhow::Result<()> {
        use snow2d::gfx::tex::TextureLoader;

        // register asset loaders
        ice.assets.add_cache::<Texture2dDrop>(TextureLoader);
        snow2d::audio::asset::register_asset_loaders(&mut ice.assets, &ice.audio.clone());

        // load type objects
        load_type_objects(ice)?;

        return Ok(());

        fn load_type_objects(ice: &mut Ice) -> anyhow::Result<()> {
            snow2d::asset::with_cache(&mut ice.assets, |cache| unsafe {
                TypeObjectStorageBuilder::begin()
                    .unwrap()
                    .register::<ActorImageType, &AssetKey<'static>>(
                        paths::types::actors::ACTOR_IMAGES,
                        cache,
                    )?
                    .register::<ActorType, &AssetKey<'static>>(
                        paths::types::actors::ACTOR_TYPES,
                        cache,
                    )?
                    .register::<DirAnimType, &AssetKey<'static>>(paths::types::ANIM_TYPES, cache)?;

                Ok(())
            })
        }
    }
}

fn gen_data(w: u32, h: u32, mut ice: Ice) -> Result<Data> {
    let mut res = init_res(&mut ice, Ui::new())?;
    let gui = self::init_world([w, h], &mut ice, &mut res.ui)?;

    let mut system = GameSystem::default();
    system.model = gui.vm.clone();
    // TODO: set up event hub

    return Ok(Data {
        system,
        ice,
        gui,
        res,
        cfg: GameConfig {
            vol: 1.0,
            shadow_cfg: ShadowConfig::Blur,
            snow_cfg: SnowConfig::Blizzard,
        },
    });

    fn init_res(ice: &mut Ice, ui: Ui) -> Result<Resources> {
        let fonts = load_fonts(ice);
        let kbd_icons = {
            let kbd_icons_tex = ice.assets.load_sync_preserve(paths::img::kbd::KBD_2)?;

            KbdIcons::new(
                kbd_icons_tex,
                &ice.assets.resolve(paths::img::kbd::KBD_2_PACK),
            )?
        };

        return Ok(Resources {
            fonts,
            kbd_icons,
            vi: VInput::new(),
            ui,
            dir_anims: Default::default(),
        });

        fn load_fonts(ice: &mut Ice) -> res::Fonts {
            ice.snow.fontbook.tex.set_size(consts::DEFAULT_FONT_SIZE);
            // line_spacing: crate::consts::DEFAULT_LINE_SPACE,

            res::Fonts {
                default: {
                    use snow2d::gfx::text::font::*;

                    let family_desc = FontFamilyDesc {
                        name: "mplus-1p".to_string(),
                        regular: FontDesc {
                            name: "mplus-1p-regular".to_string(),
                            load: include_bytes!("../assets_embedded/mplus-1p-regular.ttf")
                                .as_ref()
                                .into(),
                        },
                        bold: Some(FontDesc {
                            name: "mplus-1p-bold".to_string(),
                            load: include_bytes!("../assets_embedded/mplus-1p-bold.ttf")
                                .as_ref()
                                .into(),
                        }),
                        italic: None,
                    };

                    ice.snow.fontbook.load_family(&family_desc).unwrap()
                },
            }
        }
    }
}

fn init_world(screen_size: [u32; 2], ice: &mut Ice, ui: &mut Ui) -> anyhow::Result<Gui> {
    let (map_view, map_model) = view::map::load_tiled(paths::map::tmx::TILES, &mut ice.assets)?;

    let mut model = Model::default();
    model.map = map_model;

    let radius = [consts::FOV_R, 10];
    let map_size = model.map.size;

    let mut gui = Gui {
        vm: model,
        cam: Camera2d {
            params: Transform2dParams {
                pos: [200.0, 20.0].into(),
                scale: [1.0, 1.0].into(),
                rot: 0.0,
            },
            size: Vec2f::new(screen_size[0] as f32, screen_size[1] as f32),
        },
        cam_follow: FollowCamera2d {
            // TODO: don't hardcode. maybe use expressions considering window resizing
            sense_pads: Vec2f::new(320.0, 180.0),
            target_pads: Vec2f::new(340.0, 200.0),
            deadzone: Rect2f::new(
                0.0,
                0.0,
                map_size[0] as f32 * map_view.tiled.tile_width as f32,
                map_size[1] as f32 * map_view.tiled.tile_height as f32,
            ),
            lerp_speed: 0.1,
            is_moving: false,
        },
        map: map_view,
        shadow: Shadow::new(radius, map_size, consts::WALK_SECS, consts::FOV_EASE),
        actors: Arena::with_capacity(20),
    };

    snow2d::asset::with_cache(&mut ice.assets, |_cache| {
        load_actors(&mut gui, ui).unwrap();
    });

    // animate initial FoV:
    gui.shadow.mark_dirty();
    // just set FoV:
    // shadow.calculate(player.pos, &map.rlmap);

    return Ok(gui);

    fn load_actors(gui: &mut Gui, ui: &mut Ui) -> anyhow::Result<()> {
        // player
        // ActorSpawn::new("ika-chan")
        ActorSpawn::new("mokusei-san")
            .pos([12, 16])
            .dir(Dir8::S)
            .spawn(gui, ui)?;

        // non-player characters
        let mut spawn = ActorSpawn::new("mokusei-san");

        spawn.pos([14, 12]).dir(Dir8::W).friendly().spawn(gui, ui)?;

        spawn.pos([25, 18]).dir(Dir8::E).hostile().spawn(gui, ui)?;

        Ok(())
    }
}
