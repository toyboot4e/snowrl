/*!
SnowRL is a turn-based roguelike game
*/

#![feature(const_raw_ptr_deref)]

pub extern crate gui;
pub extern crate rokol;
pub extern crate snow2d;

pub mod consts;
pub mod paths;
pub mod states;

use std::path::PathBuf;

use anyhow::*;

use gui::prelude::*;

/// SnowRL the game
#[derive(Debug)]
pub struct SnowRl {
    pub data: Data,
    pub fsm: Fsm<Data>,
    pub world_render: WorldRenderer,
}

#[cfg(feature = "sdl2")]
mod impl_ {
    use super::*;

    use sdl2::event::Event;
    use std::time::Duration;

    impl SnowRl {
        #[inline(always)]
        pub fn event(&mut self, ev: &Event, _platform: &mut Platform) {
            self.data.ice.event(ev);
        }

        #[inline(always)]
        pub fn update(&mut self, dt: Duration, platform: &mut Platform) {
            self.pre_update(dt, platform);
            self.fsm.update(&mut self.data);
            self.post_update(dt, platform);
        }

        pub const DEFAULT_RENDER_SCHEDULE: &'static [DrawStage] = &[
            DrawStage::MapDown,
            DrawStage::UiLayer(UiLayer::Actors),
            DrawStage::UiLayer(UiLayer::OnActors),
            DrawStage::MapUp,
            DrawStage::Shadow,
            DrawStage::UiLayer(UiLayer::OnShadow),
            DrawStage::Snow,
            DrawStage::UiLayer(UiLayer::Screen),
        ];

        #[inline(always)]
        pub fn render(&mut self, dt: Duration, platform: &mut Platform) {
            // FIXME:
            let window = self.data.ice.snow.window.clone();
            self.data.ice.pre_render(window);
            DrawStage::draw_schedule(Self::DEFAULT_RENDER_SCHEDULE, self);
            self.data.ice.post_render(dt);

            self.data.ice.on_end_frame();
            rg::commit();
            platform.swap_window();
        }

        #[inline(always)]
        fn pre_update(&mut self, dt: Duration, _platform: &mut Platform) {
            self.data.ice.pre_update(dt);
            self.data.gui.update(&mut self.data.ice);
            self.data.res.vi.update(&self.data.ice.input, dt);
        }

        #[inline(always)]
        fn post_update(&mut self, dt: Duration, _platform: &mut Platform) {
            // shadow
            // FIXME: don't hard code player detection
            const PLAYER_SLOT: u32 = 0;
            let player_view = &self.data.gui.entities.get_by_slot(PLAYER_SLOT).unwrap().1;
            let player_model = &self.data.gui.vm.entities[player_view.model];

            self.data
                .gui
                .shadow
                .post_update(dt, &self.data.gui.vm.map, player_model.pos);

            // camera
            let player_pos = player_view.img.pos_world_centered(&self.data.gui.map.tiled);
            self.data.gui.cam_follow.update_follow(
                &mut self.data.gui.cam,
                player_pos,
                Vec2f::from(self.data.ice.snow.window.size_f32()),
            );

            // sprites
            self.data.res.ui.update(dt);
            self.data.res.dir_anims.update(dt, &mut self.data.res.ui);

            // renderer
            self.world_render.post_update(&self.data.gui.vm, dt);
        }

        // #[inline(always)]
        // fn debug_render(&mut self, dt: Duration, platform: &mut Platform) {
        //     //
        // }
    }
}

/// Component of rendering schedule
#[derive(Debug, Clone, Copy)]
pub enum DrawStage {
    UiLayer(UiLayer),
    /// Down parts of the map
    MapDown,
    /// Up parts of the map
    MapUp,
    Shadow,
    Snow,
    /// Clear screen with cornflower blue
    ClearScreen,
}

impl DrawStage {
    pub fn draw_schedule(schedule: &[Self], rl: &mut SnowRl) {
        for stage in schedule {
            stage.draw(rl);
        }
    }

    const PA_BLUE: rg::PassAction =
        rg::PassAction::clear_const([100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 250.0 / 255.0]);

    pub fn draw(self, app: &mut SnowRl) {
        let (data, world_render) = (&mut app.data, &mut app.world_render);
        let cam_mat = data.gui.cam.to_mat4();

        let (ice, res, world, cfg) = (&mut data.ice, &mut data.res, &mut data.gui, &data.cfg);
        let dt = ice.dt();

        match self {
            DrawStage::UiLayer(ui_layer) => {
                if ui_layer == UiLayer::Actors {
                    // FIXME: we're assuming `OnActors` is drawn actor `Actors`
                    world_render.setup_actor_nodes(world, &mut res.ui, dt);
                }

                let mut screen = ice
                    .snow
                    .screen()
                    .transform(match ui_layer.to_layer().coord {
                        CoordSystem::Screen => None,
                        CoordSystem::World => Some(cam_mat),
                    })
                    .build();

                res.ui.render_range(ui_layer.to_draw_range(), &mut screen);
            }
            DrawStage::MapDown => {
                let mut screen = ice
                    .snow
                    .screen()
                    .pa(Some(&rg::PassAction::LOAD))
                    .transform(Some(cam_mat))
                    .build();
                WorldRenderer::render_map(&mut screen, world, 0..100);
            }
            DrawStage::MapUp => {
                let mut screen = ice
                    .snow
                    .screen()
                    .pa(Some(&Self::PA_BLUE))
                    .transform(Some(cam_mat))
                    .build();
                WorldRenderer::render_map(&mut screen, world, 100..);
            }
            DrawStage::Shadow => {
                world_render.render_shadow(&mut ice.snow, world, &cfg.shadow_cfg);
            }
            DrawStage::Snow => {
                world_render.render_snow(&ice.snow.window, &ice.snow.clock, &cfg.snow_cfg);
            }
            DrawStage::ClearScreen => {
                // TODO: is this inefficient
                let _screen = ice.snow.screen().pa(Some(&Self::PA_BLUE)).build();
            }
        }
    }
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
    let mut data = self::gen_data(init.w, init.h, ice)?;
    let fsm = self::gen_fsm(&mut data)?;
    let world_render = WorldRenderer::new([init.w, init.h], &data.ice.snow.clock);

    Ok((
        platform,
        SnowRl {
            data,
            fsm,
            world_render,
        },
    ))
}

fn gen_ice(w: u32, h: u32) -> Result<Ice> {
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
            snow2d::asset::with_cache(&mut ice.assets, |cache| {
                tyobj::storage_builder()
                    .unwrap()
                    .add::<ActorImageType, &AssetKey<'static>>(
                        paths::types::actors::ACTOR_IMAGES,
                        cache,
                    )?
                    .add::<ActorType, &AssetKey<'static>>(paths::types::actors::ACTOR_TYPES, cache)?
                    .add::<DirAnimType, &AssetKey<'static>>(paths::types::ANIM_TYPES, cache)?;

                Ok(())
            })
        }
    }
}

fn gen_data(w: u32, h: u32, mut ice: Ice) -> Result<Data> {
    let mut res = init_res(&mut ice, Ui::new())?;
    let gui = init_world([w, h], &mut ice, &mut res.ui)?;

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

        fn load_fonts(ice: &mut Ice) -> Fonts {
            ice.snow.fontbook.tex.set_size(consts::DEFAULT_FONT_SIZE);
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

                    ice.snow.fontbook.load_family(&family_desc).unwrap()
                },
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
            entities: Arena::with_capacity(20),
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
}

fn gen_fsm(data: &mut Data) -> Result<Fsm<Data>> {
    let mut fsm = Fsm::default();

    fsm.insert(states::TickState::default());

    fsm.push::<states::TickState>(data);

    Ok(fsm)
}
