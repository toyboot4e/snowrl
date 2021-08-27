/*!
Map view
*/

use anyhow::*;
use std::path::Path;

use snow2d::{
    asset::{Asset, AssetCache, AssetKey},
    gfx::{
        draw::Texture2d,
        tex::{SharedSubTexture2d, Texture2dDrop},
    },
};

use model::map::MapModel;

/// View to the map model
#[derive(Debug)]
pub struct MapView {
    /// FIXME: Replace tiled map with ViewMap
    pub tiled: tiled::Map,
    pub idmap: GidTextureMap,
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
                    let img_path = tiled_dir_path.join(relative_img_path);
                    cache.load_sync(AssetKey::from_path(img_path)).unwrap()
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

pub fn load_tiled<'a>(
    tiled_path: impl Into<AssetKey<'a>>,
    cache: &mut AssetCache,
) -> Result<(MapView, MapModel)> {
    let resolved = cache.resolve(tiled_path);

    let tiled = tiled::parse_file(&resolved)?;
    let idmap = GidTextureMap::from_tiled(&resolved, &tiled, cache)?;

    let rlmap = self::rlmap_from_tiled(&tiled);

    Ok((MapView { tiled, idmap }, rlmap))
}

fn rlmap_from_tiled(tiled: &tiled::Map) -> MapModel {
    let meta = tiled
        .layers
        .iter()
        .find(|l| l.name == "meta")
        .expect("layer with name `meta` is required");

    let capacity = (tiled.width * tiled.height) as usize;
    let mut body_blocks = Vec::with_capacity(capacity);
    let mut view_blocks = Vec::with_capacity(capacity);

    let tiles = match &meta.tiles {
        tiled::LayerData::Finite(f) => f,
        tiled::LayerData::Infinite(_) => unimplemented!("tiled map infinite layer"),
    };

    // fill the blocks
    for (_y, row) in tiles.iter().enumerate() {
        for (_x, layer_tile) in row.iter().enumerate() {
            let gid = layer_tile.gid;
            if gid == 0 {
                body_blocks.push(false);
                view_blocks.push(false);
                continue;
            }

            let tileset = tiled
                .get_tileset_by_gid(gid)
                .expect("no corresponding tileset for gid?");
            let tile_ix = gid - tileset.first_gid;
            let tile = &tileset.tiles[tile_ix as usize];

            body_blocks.push(tile.properties.keys().any(|k| k == "is-body-block"));
            view_blocks.push(tile.properties.keys().any(|k| k == "is-view-block"));
        }
    }

    let size = [tiled.width as usize, tiled.height as usize];
    MapModel {
        size,
        body_blocks,
        view_blocks,
    }
}
