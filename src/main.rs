//! TODO: remove debug/error log on release build?
//! TODO: inspect Pool/Anim and see if there's garbage

use rokol::app as ra;

use rlbox::{
    rl::grid2d::*,
    view::{
        camera::{Camera2d, FollowCamera2d, TransformParams2d},
        map::TiledRlMap,
        shadow::Shadow,
    },
};

use grue2d::data::{
    resources::{Resources, Ui, VInput},
    rogue::Rogue,
    world::{actor::*, World},
};

use snowrl::{
    init,
    prelude::*,
    states,
    utils::{consts, paths},
    SnowRl,
};

fn main() -> ra::glue::Result {
    env_logger::init();

    let rokol = ra::glue::Rokol {
        w: 1280,
        h: 720,
        use_high_dpi: false,
        // TODO: text-only high DPI game application (other items should be scaled)
        // use_high_dpi: true,
        title: "SnowRL".to_string(),
        ..Default::default()
    };

    grue2d::run(rokol, |rokol| SnowRl::new(self::new_game(rokol)))
}

fn new_game(rokol: ra::glue::Rokol) -> GrueRl {
    let title = rokol.title;

    // create our game context
    let mut data = {
        let mut ice = Ice::new(title, unsafe { Snow2d::new() });
        init::init_assets(&mut ice).unwrap();

        let mut ui = Ui::new();
        let world = self::init_world(&mut ice, &mut ui).unwrap();
        let fonts = init::load_fonts(&mut ice);

        Data {
            ice,
            world,
            res: Resources {
                fonts,
                vi: VInput::new(),
                ui: Ui::new(),
            },
            rogue: Rogue::new(),
        }
    };

    // create our control
    let fsm = {
        let mut fsm = grue2d::fsm::Fsm::default();

        fsm.insert_default::<states::Roguelike>();
        fsm.insert_default::<states::Animation>();

        // fsm.insert(states::Title::new(&mut gl.ice));
        fsm.insert(states::Title::new(&mut data.ice, &mut data.res.ui));

        fsm.insert(states::PlayScript::new(&mut data.ice.assets));

        fsm.push::<states::Roguelike>(&mut data);
        fsm.push::<states::Title>(&mut data);

        fsm
    };

    GrueRl::new(data, fsm)
}

fn init_world(ice: &mut Ice, ui: &mut Ui) -> anyhow::Result<World> {
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
            size: ra::size_f().into(),
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
    // TODO: ActorBuilder::from_type

    // player
    ActorSpawn::new("ika-chan")
        .pos([14, 10])
        .dir(Dir8::S)
        .spawn(world, ui)?;

    // non-player characters
    let mut spawn = ActorSpawn::new("mokusei-san");

    spawn.pos([14, 12]).dir(Dir8::W).spawn(world, ui)?;
    spawn.pos([25, 18]).dir(Dir8::E).spawn(world, ui)?;

    Ok(())
}
