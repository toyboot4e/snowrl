/*!
SnowRL initialization module
*/

use snow2d::{asset::StaticAssetKey, utils::typeobj::TypeObjectStorageBuilder, Ice};

use rlbox::view::actor::ActorImageDesc;

use grue2d::rl::world::actor::*;

use crate::prelude::*;

pub fn init_assets(ice: &mut Ice) -> anyhow::Result<()> {
    ice.assets
        .add_cache::<Texture2dDrop>(AssetCacheT::new(TextureLoader));
    snow2d::audio::asset::register_asset_loaders(&mut ice.assets, &ice.audio.clone());
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
            .register::<ActorImageDesc, StaticAssetKey>(paths::actors::ACTOR_IMAGES)?
            .register::<ActorType, StaticAssetKey>(paths::actors::ACTOR_TYPES)?;
    }

    unsafe {
        snow2d::asset::AssetDeState::end().unwrap();
    }

    Ok(())
}

pub fn load_fonts(ice: &mut Ice) -> Fonts {
    ice.snow
        .fontbook
        .tex
        .set_size(consts::DEFAULT_FONT_SIZE);
    // line_spacing: crate::consts::DEFAULT_LINE_SPACE,

     Fonts {
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
    }
}
