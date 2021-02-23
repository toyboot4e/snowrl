/*!
Roguelike map with Tiled backend

TODO: tiled-free implementation
*/

use crate::rl::{grid2d::Vec2i, shadow::OpacityMap};

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

/// FoV
impl OpacityMap for RlMap {
    fn is_opaque(&self, pos: Vec2i) -> bool {
        self.is_view_blocked(pos)
    }

    fn contains(&self, pos: Vec2i) -> bool {
        <Self>::contains(self, pos)
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
