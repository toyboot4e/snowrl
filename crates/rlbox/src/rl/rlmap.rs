//! Roguelike map with Tiled backend

use {
    anyhow::{Context, Result},
    snow2d::{
        asset::{self, Asset, AssetCacheT},
        gfx::{
            batcher::draw::Texture2d,
            tex::{SharedSubTexture2d, Texture2dDrop},
        },
    },
    std::path::Path,
};

use crate::rl::{fov::OpacityMap, grid2d::Vec2i};

/// Bundles tiled map and roguelike map data
#[derive(Debug)]
pub struct TiledRlMap {
    pub tiled: tiled::Map,
    pub rlmap: RlMap,
    pub idmap: GidTextureMap,
}

/// fs
impl TiledRlMap {
    pub fn from_tiled_path(
        tiled_path: impl AsRef<Path>,
        cache: &mut AssetCacheT<Texture2dDrop>,
    ) -> Result<Self> {
        let tiled_path = unsafe { asset::path(tiled_path) };

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
///
/// Considers layer with name "collision".
#[derive(Debug)]
pub struct RlMap {
    pub size: [usize; 2],
    /// True if collidges
    pub blocks: Vec<bool>,
}

impl RlMap {
    pub fn contains(&self, pos: impl Into<Vec2i>) -> bool {
        let pos = pos.into();
        let (x, y) = (pos.x, pos.y);
        !(x < 0 || y < 0 || self.size[0] as i32 <= x || self.size[1] as i32 <= y)
    }

    /// Returns if the position is blocked or outsize of the map
    pub fn is_blocked(&self, pos: impl Into<Vec2i>) -> bool {
        let pos = pos.into();
        if !self.contains(pos) {
            return true;
        }

        let ix = pos.x + self.size[0] as i32 * pos.y;
        self.blocks[ix as usize]
    }
}

/// IO
impl RlMap {
    pub fn from_tiled(tiled: &tiled::Map) -> Self {
        let collision = tiled
            .layers
            .iter()
            .find(|l| l.name == "collision")
            .expect("layer with name `collision` is required");

        // extract collision data from tiled map data
        let mut blocks = Vec::with_capacity((tiled.width * tiled.height) as usize);
        let tiles = match &collision.tiles {
            tiled::LayerData::Finite(f) => f,
            tiled::LayerData::Infinite(_) => unimplemented!("tiled map infinite layer"),
        };

        for (_y, row) in tiles.iter().enumerate() {
            for (_x, tile) in row.iter().enumerate() {
                // if any tile is placed, it's blocking
                blocks.push(tile.gid != 0);
            }
        }

        let size = [tiled.width as usize, tiled.height as usize];
        Self { size, blocks }
    }
}

/// FoV
impl OpacityMap for RlMap {
    fn is_opaque(&self, pos: Vec2i) -> bool {
        !self.contains(pos) || self.is_blocked(pos)
    }

    fn contains(&self, pos: Vec2i) -> bool {
        <Self>::contains(self, pos)
    }
}

/// Maps global tile id to sub texture
#[derive(Debug, Clone)]
pub struct GidTextureMap {
    pub imgs: Vec<TilesetImageSpan>,
    tile_size: [u32; 2],
}

#[derive(Debug, Clone)]
pub struct TilesetImageSpan {
    first_gid: u32,
    tex: Asset<Texture2dDrop>,
}

impl GidTextureMap {
    // TODO: don't use anyhow
    pub fn from_tiled(
        tiled_file_path: &Path,
        tiled: &tiled::Map,
        cache: &mut AssetCacheT<Texture2dDrop>,
    ) -> anyhow::Result<Self> {
        let tiled_dir_path = tiled_file_path
            .parent()
            .context("GidTextureMap from_tile path error")?;

        let tile_size = [tiled.tile_width, tiled.tile_height];

        let mut imgs = Vec::with_capacity(1);
        for tileset in &tiled.tilesets {
            imgs.push(TilesetImageSpan {
                first_gid: tileset.first_gid,
                tex: {
                    let relative_img_path = &tileset.images[0].source;
                    let img_path = tiled_dir_path.join(relative_img_path);

                    cache.load_sync(&img_path).unwrap()
                },
            });
        }

        Ok(Self { imgs, tile_size })
    }

    pub fn gid_to_tile(&self, gid: u32) -> Option<SharedSubTexture2d> {
        if gid == 0 {
            return None;
        }

        for span in self.imgs.iter().rev() {
            if gid < span.first_gid {
                continue;
            }

            let id = gid - span.first_gid;
            let tex_size = span.tex.get().unwrap().sub_tex_size();

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
