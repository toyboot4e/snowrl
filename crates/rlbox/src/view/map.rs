/*!
Map view
*/

use {
    anyhow::*,
    snow2d::{
        asset::{self, Asset, AssetCache, AssetKey},
        gfx::{
            draw::Texture2d,
            tex::{SharedSubTexture2d, Texture2dDrop},
        },
    },
    std::path::Path,
};

use crate::rl::rlmap::RlMap;

/// Bundle of Tiled map and internal roguelike map data
#[derive(Debug)]
pub struct TiledRlMap {
    pub tiled: tiled::Map,
    pub rlmap: RlMap,
    pub idmap: GidTextureMap,
}

/// fs
impl TiledRlMap {
    pub fn new(tiled_path: impl AsRef<Path>, cache: &mut AssetCache) -> Result<Self> {
        let tiled_path = asset::path(tiled_path);
        let tiled = tiled::parse_file(&tiled_path)?;

        let rlmap = RlMap::from_tiled(&tiled);
        let idmap = GidTextureMap::from_tiled(&tiled_path, &tiled, cache)?;

        Ok(Self {
            tiled,
            rlmap,
            idmap,
        })
    }
}

/// Maps Tiled's GID (global tile id) to a texture
#[derive(Debug, Clone)]
pub struct GidTextureMap {
    spans: Vec<GidTextureSpan>,
    tile_size: [u32; 2],
}

/// Span of Tiled's gid (global tile id) that uses one texture
#[derive(Debug, Clone)]
struct GidTextureSpan {
    first_gid: u32,
    tex: Asset<Texture2dDrop>,
}

impl GidTextureMap {
    // TODO: don't use anyhow
    pub fn from_tiled(
        tmx_file_path: &Path,
        tiled: &tiled::Map,
        cache: &mut AssetCache,
    ) -> anyhow::Result<Self> {
        let tiled_dir_path = tmx_file_path
            .parent()
            .context("GidTextureMap from_tile path error")?;

        let tile_size = [tiled.tile_width, tiled.tile_height];

        let mut spans = Vec::with_capacity(1);
        for tileset in &tiled.tilesets {
            spans.push(GidTextureSpan {
                first_gid: tileset.first_gid,
                tex: {
                    // NOTE: this value can either be relative path from tmx file or tsx file
                    let relative_img_path = &tileset.images[0].source;
                    log::trace!("{:?}", relative_img_path);
                    let img_path = tiled_dir_path.join(relative_img_path);

                    cache.load_sync(AssetKey::new(&img_path)).unwrap()
                },
            });
        }

        Ok(Self { spans, tile_size })
    }

    pub fn gid_to_tile(&self, gid: u32) -> Option<SharedSubTexture2d> {
        if gid == 0 {
            return None;
        }

        for span in self.spans.iter().rev() {
            if gid < span.first_gid {
                continue;
            }

            let id = gid - span.first_gid;
            let tex_size = span.tex.get().unwrap().sub_tex_size_unscaled();

            let n_cols = tex_size[0] as u32 / self.tile_size[0];
            let src_grid_x = id % n_cols;
            let src_grid_y = id / n_cols;

            return Some(SharedSubTexture2d {
                tex: span.tex.clone(),
                uv_rect: [
                    self.tile_size[0] as f32 * src_grid_x as f32 / tex_size[0],
                    self.tile_size[1] as f32 * src_grid_y as f32 / tex_size[1],
                    self.tile_size[0] as f32 / tex_size[0],
                    self.tile_size[1] as f32 / tex_size[1],
                ],
            });
        }

        None
    }
}
