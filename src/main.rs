//! TODO: filter debug/error log on release build

use rokol::Rokol;
use std::{fs, io::prelude::*};

use snow2d::{
    asset::StaticAssetKey,
    ui::{CoordSystem, Layer},
    utils::type_object::TypeObject,
};

use rlbox::{
    rl::grid2d::*,
    view::{
        actor::{ActorImage, ActorImageDesc, DirAnimKind},
        camera::*,
        map::TiledRlMap,
        shadow::Shadow,
    },
};

use grue2d::{
    render::WorldRenderer,
    rl::{
        turn::anim::AnimPlayer,
        world::{actor::*, World},
    },
    Fonts, VInput,
};

use snowrl::{
    prelude::*,
    states,
    utils::{consts, paths},
    SnowRl,
};

fn main() -> rokol::Result {
    env_logger::init();

    let rokol = rokol::Rokol {
        w: 1280,
        h: 720,
        use_high_dpi: false,
        // TODO: text-only high DPI game application (other items should be scaled)
        // use_high_dpi: true,
        title: "SnowRL".to_string(),
        ..Default::default()
    };

    grue2d::run(rokol, |rokol| SnowRl {
        grue: self::new_game(rokol),
        plugin: {
            let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            grue2d::hot_crate::HotLibrary::load(
                root.join("Cargo.toml"),
                root.join("crates/plugins/Cargo.toml"),
            )
            .unwrap()
        },
    })
}

fn new_game(rokol: Rokol) -> GlueRl {
    let title = rokol.title;

    let mut ice = Ice::new(title, unsafe { Snow2d::new() });

    ice.snow
        .fontbook
        .tex
        .set_size(crate::consts::DEFAULT_FONT_SIZE);
    // line_spacing: crate::consts::DEFAULT_LINE_SPACE,

    let fonts = Fonts {
        default: {
            use snow2d::gfx::text::font::*;
            let family_desc = FontSetDesc {
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
    };

    // create our game context
    let mut gl = {
        ice.assets
            .add_cache::<Texture2dDrop>(AssetCacheT::new(TextureLoader));

        audio::asset::register_asset_loaders(&mut ice.assets, &ice.audio.clone());
        self::load_type_objects(&mut ice);

        let world = self::init_world(&mut ice).unwrap();

        Global {
            world,
            world_render: WorldRenderer::default(),
            ice,
            fonts,
            vi: VInput::new(),
            ui: Default::default(),
            anims: AnimPlayer::default(),
            script_to_play: None,
        }
    };

    // create our control
    let fsm = {
        let mut fsm = grue2d::fsm::Fsm::default();

        fsm.insert_default::<states::Roguelike>();
        fsm.insert_default::<states::Animation>();

        // fsm.insert(states::Title::new(&mut gl.ice));
        fsm.insert(states::Title::new(&mut gl.ice, &mut gl.ui));

        let world_ui_layer_ix = gl.ui.layers.insert(Layer::new(CoordSystem::World));
        fsm.insert(states::PlayScript::new(
            &mut gl.ice.assets,
            world_ui_layer_ix,
        ));

        fsm.push::<states::Roguelike>(&mut gl);
        fsm.push::<states::Title>(&mut gl);

        fsm
    };

    GlueRl::new(gl, fsm)
}

/// TODO: use `init.rs`
fn load_type_objects(ice: &mut Ice) {
    unsafe {
        snow2d::asset::AssetDeState::start(&mut ice.assets).unwrap();
    }

    use snow2d::utils::type_object::TypeObjectStorage;
    TypeObjectStorage::init().unwrap();
    {
        log::trace!("registering type object storage for type `ActorImageDesc`");
        TypeObjectStorage::register_type_objects::<ActorImageDesc, StaticAssetKey>(
            crate::paths::actors::ACTOR_IMAGES,
        )
        .unwrap();

        log::trace!("registering type object storage for type `ActorType`");
        TypeObjectStorage::register_type_objects::<ActorType, StaticAssetKey>(
            crate::paths::actors::ACTOR_TYPES,
        )
        .unwrap();
    }

    unsafe {
        snow2d::asset::AssetDeState::end().unwrap();
    }

    log::trace!("loaded type objects");
}

fn init_world(ice: &mut Ice) -> anyhow::Result<World> {
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
            size: rokol::app::size_f().into(),
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

    self::load_actors(&mut world, ice)?;

    // just set FoV:
    // shadow.calculate(player.pos, &map.rlmap);
    // or animate initial FoV:
    world.shadow.mark_dirty();

    Ok(world)
}

fn load_actors(w: &mut World, ice: &mut Ice) -> anyhow::Result<()> {
    unsafe {
        snow2d::asset::AssetDeState::start(&mut ice.assets).unwrap();
    }

    let cache = ice.assets.cache_mut::<Texture2dDrop>().unwrap();

    // player
    let mut player: Actor = ActorType::from_type_key(&"ika-chan".into())?.to_actor();
    player.pos = [14, 10].into();
    player.img.warp(player.pos, player.dir);
    w.entities.insert(player);

    unsafe {
        snow2d::asset::AssetDeState::end().unwrap();
    }

    // non-player characters

    let tex = cache.load_sync(paths::img::pochi::WHAT).unwrap();
    let img = ActorImage::from_desc(
        &ActorImageDesc {
            tex,
            kind: DirAnimKind::Dir8,
        },
        ez::EasedDtDesc {
            target: consts::WALK_SECS,
            ease: consts::WALK_EASE,
        },
        [0, 0].into(),
        Dir8::S,
    );

    w.entities.insert({
        let mut actor = Actor {
            pos: [14, 12].into(),
            dir: Dir8::S,
            img: img.clone(),
            stats: ActorStats {
                hp: 100,
                atk: 50,
                def: 20,
            },
        };
        actor.img.warp(actor.pos, actor.dir);
        actor
    });

    w.entities.insert({
        let mut actor = Actor {
            pos: [25, 18].into(),
            dir: Dir8::S,
            img: img.clone(),
            stats: ActorStats {
                hp: 100,
                atk: 50,
                def: 20,
            },
        };
        actor.img.warp(actor.pos, actor.dir);
        actor
    });

    Ok(())
}
