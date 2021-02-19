/*!
Roguelike map with Tiled backend

TODO: tiled-free implementation
*/

use {
    anyhow::{Context, Result},
    snow2d::{
        asset::{self, Asset, AssetCacheT, AssetKey},
        gfx::{
            draw::Texture2d,
            tex::{SharedSubTexture2d, Texture2dDrop},
        },
    },
    std::path::Path,
};

use crate::rl::{fov::OpacityMap, grid2d::Vec2i};

/// Bundle of Tiled map and internal roguelike map data
#[derive(Debug)]
pub struct TiledRlMap {
    pub tiled: tiled::Map,
    pub rlmap: RlMap,
    pub idmap: GidTextureMap,
}

/// fs
impl TiledRlMap {
    pub fn new(
        tiled_path: impl AsRef<Path>,
        cache: &mut AssetCacheT<Texture2dDrop>,
    ) -> Result<Self> {
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

/// Roguelike map data
#[derive(Debug)]
pub struct RlMap {
    pub size: [usize; 2],
    /// True if it's physical block
    pub body_blocks: Vec<bool>,
    /// True if it's view block
    pub view_blocks: Vec<bool>,
}

impl RlMap {
    pub fn contains(&self, pos: impl Into<Vec2i>) -> bool {
        let pos = pos.into();
        let (x, y) = (pos.x, pos.y);
        // not outsize of the map
        !(x < 0 || y < 0 || self.size[0] as i32 <= x || self.size[1] as i32 <= y)
    }

    pub fn is_body_blocked(&self, pos: impl Into<Vec2i>) -> bool {
        let pos = pos.into();

        if !self.contains(pos) {
            return true;
        }

        let ix = pos.x + self.size[0] as i32 * pos.y;
        self.body_blocks[ix as usize]
    }

    pub fn is_view_blocked(&self, pos: impl Into<Vec2i>) -> bool {
        let pos = pos.into();

        if !self.contains(pos) {
            return true;
        }

        let ix = pos.x + self.size[0] as i32 * pos.y;
        self.view_blocks[ix as usize]
    }
}

/// IO
impl RlMap {
    /// Load data requiring "meta" layer
    pub fn from_tiled(tiled: &tiled::Map) -> Self {
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
        Self {
            size,
            body_blocks,
            view_blocks,
        }
    }
}

/// FoV
impl OpacityMap for RlMap {
    fn is_opaque(&self, pos: Vec2i) -> bool {
        self.is_view_blocked(pos)
    }

    fn contains(&self, pos: Vec2i) -> bool {
        <Self>::contains(self, pos)
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
        cache: &mut AssetCacheT<Texture2dDrop>,
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
