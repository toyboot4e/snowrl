/*!
World/resource initialization
*/

use snow2d::{
    asset::{Asset, AssetKey},
    ui::Ui,
    utils::tyobj::TypeObjectStorageBuilder,
    Ice,
};

use rlbox::{
    rl::grid2d::*,
    view::{
        actor::ActorImageType,
        anim::DirAnimType,
        camera::{Camera2d, FollowCamera2d, TransformParams2d},
        map::TiledRlMap,
        shadow::Shadow,
    },
};

use grue2d::{
    game::data::world::{actor::*, World},
    markup::KbdIcons,
};

use crate::prelude::*;

pub fn init_assets(ice: &mut Ice) -> anyhow::Result<()> {
    // register asset loaders
    ice.assets.add_cache::<Texture2dDrop>(TextureLoader);
    snow2d::audio::asset::register_asset_loaders(&mut ice.assets, &ice.audio.clone());

    // load type objects
    self::load_type_objects(ice)?;

    Ok(())
}

fn load_type_objects(ice: &mut Ice) -> anyhow::Result<()> {
    unsafe {
        snow2d::asset::AssetDeState::start(&mut ice.assets).unwrap();
    }

    unsafe {
        TypeObjectStorageBuilder::begin()
            .unwrap()
            .register::<ActorImageType, &AssetKey<'static>>(paths::types::actors::ACTOR_IMAGES)?
            .register::<ActorType, &AssetKey<'static>>(paths::types::actors::ACTOR_TYPES)?
            .register::<DirAnimType, &AssetKey<'static>>(paths::types::ANIM_TYPES)?;
    }

    unsafe {
        snow2d::asset::AssetDeState::end().unwrap();
    }

    Ok(())
}

pub fn load_fonts(ice: &mut Ice) -> Fonts {
    ice.snow.fontbook.tex.set_size(consts::DEFAULT_FONT_SIZE);
    // line_spacing: crate::consts::DEFAULT_LINE_SPACE,

    Fonts {
        default: {
            use snow2d::gfx::text::font::*;

            let family_desc = FontFamilyDesc {
                name: "mplus-1p".to_string(),
                regular: FontDesc {
                    name: "mplus-1p-regular".to_string(),
                    load: include_bytes!("../../../../assets_embedded/mplus-1p-regular.ttf")
                        .as_ref()
                        .into(),
                },
                bold: Some(FontDesc {
                    name: "mplus-1p-bold".to_string(),
                    load: include_bytes!("../../../../assets_embedded/mplus-1p-bold.ttf")
                        .as_ref()
                        .into(),
                }),
                italic: None,
            };

            ice.snow.fontbook.load_family(&family_desc).unwrap()
        },
    }
}

pub fn init_world(screen_size: [u32; 2], ice: &mut Ice, ui: &mut Ui) -> anyhow::Result<World> {
    let map = TiledRlMap::new(paths::map::tmx::TILES, &mut ice.assets)?;

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
    // player
    // ActorSpawn::new("ika-chan")
    ActorSpawn::new("mokusei-san")
        .pos([12, 16])
        .dir(Dir8::S)
        .spawn(world, ui)?;

    // non-player characters
    let mut spawn = ActorSpawn::new("mokusei-san");

    spawn
        .pos([14, 12])
        .dir(Dir8::W)
        .friendly()
        .spawn(world, ui)?;

    spawn
        .pos([25, 18])
        .dir(Dir8::E)
        .hostile()
        .spawn(world, ui)?;

    Ok(())
}
