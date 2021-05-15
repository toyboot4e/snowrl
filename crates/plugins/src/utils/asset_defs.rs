/*!
Hard-coded asset definitions
*/

use snow2d::{
    asset::{AssetCacheAny, Result},
    gfx::tex::*,
};

use crate::utils::paths;

pub trait AssetDef {
    type Target;
    fn load(assets: &mut AssetCacheAny) -> Result<Self::Target>;
}

pub mod title {
    use super::*;

    pub struct Choices;

    impl AssetDef for Choices {
        type Target = [SpriteData; 3];

        fn load(assets: &mut AssetCacheAny) -> Result<Self::Target> {
            let tex = assets.load_sync(paths::img::title::CHOICES)?;
            let unit = 1.0 / 3.0;

            let mut s = SpriteData::builder(tex.clone());
            s.uv_rect([0.0, unit * 0.0, 1.0, unit]).scales([0.5, 0.5]);

            Ok([
                s.uv_rect([0.0, unit * 0.0, 1.0, unit]).build(),
                s.uv_rect([0.0, unit * 1.0, 1.0, unit]).build(),
                s.uv_rect([0.0, unit * 2.0, 1.0, unit]).build(),
            ])
        }
    }

    pub struct Logo;

    impl AssetDef for Logo {
        type Target = SpriteData;
        fn load(assets: &mut AssetCacheAny) -> Result<Self::Target> {
            Ok({
                let mut s = SpriteData::from_tex(assets.load_sync(paths::img::title::SNOWRL)?);
                s.scales = [0.5, 0.5];
                s
            })
        }
    }
}
