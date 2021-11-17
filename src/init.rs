use std::{fs, path::PathBuf};

use anyhow::*;

use gui::{content::PlayerAi, prelude::*};
use snow2d::asset;

use crate::{consts, paths, states, SnowRl};

/// Create a window and initialize the game
pub fn init() -> Result<(Platform, SnowRl)> {
    let asset_root = {
        // FIXME: Consider release build
        let proj_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = PathBuf::from(proj_root)
            .join("assets")
            .canonicalize()
            .unwrap();
        asset::AssetRoot::new(path)
    };

    let init: rokol::glue::sdl::Init = {
        let path = asset_root.resolve("config/init.ron");
        let bytes = fs::read(path).expect("unable to read init config");
        ron::de::from_bytes(&bytes).unwrap()
    };

    let platform = init
        .init(|w| {
            w.position_centered();
            // w.allow_hidhdpi();
        })
        .map_err(Error::msg)?;

    // ****************************************
    // Disable text input, including IME. This is important for constant FPS
    // (At least on my mac for unknown reason)
    // ****************************************
    platform.vid.text_input().stop();

    let ice = self::gen_ice(init.w, init.h, asset_root)?;
    let mut data = self::gen_data(init.w, init.h, ice, &platform)?;
    let fsm = self::gen_fsm(&mut data)?;
    let world_render = WorldRenderer::new([init.w, init.h], &data.ice.gfx.clock);

    Ok((
        platform,
        SnowRl {
            god: data,
            fsm,
            w_rdr: world_render,
        },
    ))
}

fn gen_ice(w: u32, h: u32, asset_root: asset::AssetRoot) -> Result<Ice> {
    let mut ice = {
        let snow = unsafe {
            Snow2d::new(WindowState {
                w,
                h,
                // TODO: remove the magic scaling number
                // dpi_scale: [2.0, 2.0],
                dpi_scale: [1.0, 1.0],
            })
        };

        Ice::new(snow, asset_root)
    };

    init_assets(&mut ice)?;

    return Ok(ice);

    fn init_assets(ice: &mut Ice) -> anyhow::Result<()> {
        // load type objects
        load_type_objects(ice)?;

        return Ok(());

        fn load_type_objects(ice: &mut Ice) -> anyhow::Result<()> {
            tyobj::storage_builder(&mut ice.assets)
                .unwrap()
                .add::<ActorImageType, &AssetKey<'static>>(paths::types::actors::ACTOR_IMAGES)?
                .add::<ActorType, &AssetKey<'static>>(paths::types::actors::ACTOR_TYPES)?
                .add::<DirAnimType, &AssetKey<'static>>(paths::types::ANIM_TYPES)?;

            Ok(())
        }
    }
}

fn gen_data(w: u32, h: u32, mut ice: Ice, platform: &Platform) -> Result<God> {
    let mut res = init_res(&mut ice, Ui::new())?;
    let gui = init_world([w, h], &mut ice, &mut res.ui)?;

    let mdl = gui.vm.clone();
    let mut system = GameSystem::new(mdl);
    init_system(&mut system);

    return Ok(God {
        sys: system,
        ice,
        gui,
        res,
        cfg: GameConfig {
            vol: 1.0,
            shadow_cfg: ShadowConfig::Blur,
            snow_cfg: SnowConfig::Blizzard,
        },
        #[cfg(debug_assertions)]
        imgui: {
            let display_size = [w as f32, h as f32];
            gui::debug::create_backend(display_size, platform)?
        },
        #[cfg(debug_assertions)]
        debug_states: Default::default(),
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

        fn load_fonts(ice: &mut Ice) -> Fonts {
            ice.gfx.fontbook.tex.set_size(consts::DEFAULT_FONT_SIZE);
            // line_spacing: crate::consts::DEFAULT_LINE_SPACE,

            Fonts {
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

                    ice.gfx.fontbook.load_family(&family_desc).unwrap()
                },
            }
        }
    }

    /// Sets up `EventHub` and `AiHub`
    fn init_system(sys: &mut GameSystem) {
        // event handling
        sys.hub = EventHub::build(|b| {
            b.mutate(model::evs::builder_plugin)
                .mutate(gui::content::builder_plugin);
        });

        // AIs
        sys.ais.add(PlayerAi::AI, Box::new(PlayerAi::logic));
    }

    fn init_world(screen_size: [u32; 2], ice: &mut Ice, ui: &mut Ui) -> anyhow::Result<Gui> {
        let (map_view, map_mdl) = view::map::load_tiled(paths::map::tmx::TILES, &mut ice.assets)?;

        let mut mdl = Model::default();
        mdl.map = map_mdl;

        let radius = [consts::FOV_R, 10];
        let map_size = mdl.map.size;

        let mut gui = Gui {
            vm: mdl,
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

        snow2d::asset::guarded(&mut ice.assets, |_cache| {
            load_actors(&mut gui, ui).unwrap();
        });

        // animate initial FoV:
        gui.shadow.mark_dirty();
        // just set FoV:
        // shadow.calculate(player.pos, &map.rlmap);

        return Ok(gui);

        fn load_actors(gui: &mut Gui, ui: &mut Ui) -> anyhow::Result<()> {
            // player
            ActorSpawn::new("ika-chan")
                // ActorSpawn::new("mokusei-san")
                .pos([12, 16])
                .dir(Dir8::S)
                .spawn_to_gui(gui, ui)?;

            // non-player characters
            let mut spawn = ActorSpawn::new("mokusei-san");

            spawn
                .pos([14, 12])
                .dir(Dir8::W)
                .friendly()
                .spawn_to_gui(gui, ui)?;
            spawn
                .pos([25, 18])
                .dir(Dir8::E)
                .hostile()
                .spawn_to_gui(gui, ui)?;

            Ok(())
        }
    }
}

fn gen_fsm(god: &mut God) -> Result<Fsm<God>> {
    let mut fsm = Fsm::default();

    fsm.insert(states::TickState::default());
    fsm.insert(states::GuiSync::default());
    fsm.insert(states::PlayerState::default());

    fsm.push::<states::TickState>(god);

    Ok(fsm)
}
