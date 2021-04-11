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

use rlbox::{
    rl::grid2d::*,
    view::{
        camera::{Camera2d, FollowCamera2d, TransformParams2d},
        map::TiledRlMap,
        shadow::Shadow,
    },
};

use grue2d::{
    ctrl::Control,
    data::{
        res::{Resources, Ui, VInput},
        world::{actor::*, World},
    },
    platform::PlatformLifetime,
};

use snowrl::{
    init,
    prelude::*,
    states,
    utils::{consts, paths},
    SnowRl,
};

fn main() -> Result<()> {
    env_logger::init();

    let init = grue2d::platform::Init {
        title: "SnowRL".to_string(),
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

    let mut game = self::new_game(&init, &platform);
    game.data.ice.audio.set_global_volume(0.0);
    let app = SnowRl::new(game);

    grue2d::platform::run(platform, app)
}

fn new_game(init: &grue2d::platform::Init, platform: &PlatformLifetime) -> GrueRl {
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
        let world = self::init_world(screen_size, &mut ice, &mut ui).unwrap();
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

    let size = [platform.win.size().0, platform.win.size().1];
    GrueRl::new(size, data, fsm)
}

fn init_world(screen_size: [u32; 2], ice: &mut Ice, ui: &mut Ui) -> anyhow::Result<World> {
    let map = TiledRlMap::new(
        paths::map::tmx::TILES,
        ice.assets.cache_mut::<Texture2dDrop>().unwrap(),
    )?;

    let radius = [consts::FOV_R, 10];
    let map_size = map.rlmap.size;

    let mut world = World {
        cam: Camera2d {
            params: TransformParams2d {
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
                map_size[0] as f32 * map.tiled.tile_width as f32,
                map_size[1] as f32 * map.tiled.tile_height as f32,
            ),
            lerp_speed: 0.1,
            is_moving: false,
        },
        map,
        shadow: Shadow::new(radius, map_size, consts::WALK_SECS, consts::FOV_EASE),
        entities: Arena::with_capacity(20),
    };

    // Be sure to set [`AssetDeState`] while we're loading assets
    unsafe {
        snow2d::asset::AssetDeState::start(&mut ice.assets).unwrap();
    }

    self::load_actors(&mut world, ui)?;

    unsafe {
        snow2d::asset::AssetDeState::end().unwrap();
    }

    // animate initial FoV:
    world.shadow.mark_dirty();
    // just set FoV:
    // shadow.calculate(player.pos, &map.rlmap);

    Ok(world)
}

fn load_actors(world: &mut World, ui: &mut Ui) -> anyhow::Result<()> {
    // TODO: use RON
    let layer = ui.layer_mut(UiLayer::Actors);

    // player
    ActorSpawn::new("ika-chan")
        .pos([14, 10])
        .dir(Dir8::S)
        .spawn(world, layer)?;

    // non-player characters
    let mut spawn = ActorSpawn::new("mokusei-san");

    spawn
        .pos([14, 12])
        .dir(Dir8::W)
        .friendly()
        .spawn(world, layer)?;

    spawn
        .pos([25, 18])
        .dir(Dir8::E)
        .hostile()
        .spawn(world, layer)?;

    Ok(())
}
