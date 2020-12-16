//! Tiled map rendering

use {anf::prim::*, std::cmp, tiled::LayerData};

use crate::rl::{
    fov::*,
    fow::*,
    grid2d::{Rect2i, Vec2i, Vec2u},
    rlmap::GidTextureMap,
};

// --------------------------------------------------------------------------------
// Coordinates

/// World coordinates to tile coordinates flooring remaning pixels
pub fn w2t_floor(w: impl Into<Vec2f>, tiled: &tiled::Map) -> Vec2i {
    let w = w.into();
    let x = w.x as u32 / tiled.tile_width;
    let y = w.y as u32 / tiled.tile_height;
    Vec2i::new(x as i32, y as i32)
}

/// World coordinates to tile coordinates rounding up remaning pixels
pub fn w2t_round_up(w: impl Into<Vec2f>, tiled: &tiled::Map) -> Vec2i {
    let w = w.into();
    let x = (w.x as u32 + tiled.tile_width - 1) / tiled.tile_width;
    let y = (w.y as u32 + tiled.tile_width - 1) / tiled.tile_height;
    Vec2i::new(x as i32, y as i32)
}

// --------------------------------------------------------------------------------
// Rendering

/// Renders a tiled map in a bounds in world coordinates
pub fn render_tiled(
    dcx: &mut DrawContext,
    tiled: &tiled::Map,
    idmap: &GidTextureMap,
    px_bounds: impl Into<Rect2f>,
) {
    let px_bounds: Rect2f = px_bounds.into();
    let grid_bounds = self::grid_bounds_from_pixel_bounds(tiled, &px_bounds);
    let (ys, xs) = self::visible_cells_from_grid_bounds(&grid_bounds);

    let mut pass = dcx.batch();
    for layer in tiled.layers.iter().filter(|l| l.visible) {
        render_tiled_layer(&mut pass, tiled, layer, idmap, px_bounds.left_up(), ys, xs);
    }
}

fn render_tiled_layer(
    pass: &mut BatchPass<'_>,
    tiled: &tiled::Map,
    layer: &tiled::Layer,
    idmap: &GidTextureMap,
    offset: Vec2f,
    ys: [u32; 2],
    xs: [u32; 2],
) {
    let tile_size = Vec2u::new(tiled.tile_width, tiled.tile_height);
    let tiles = match layer.tiles {
        LayerData::Finite(ref f) => f,
        LayerData::Infinite(_) => unimplemented!("tiled map infinite layer"),
    };

    for y in ys[0]..ys[1] {
        for x in xs[0]..xs[1] {
            let tile = tiles[y as usize][x as usize];

            let texture = match idmap.gid_to_tile(tile.gid) {
                Some(t) => t,
                None => continue,
            };

            pass.push(&texture).dest_rect_px([
                (
                    (x as i32 * tile_size.x as i32 - offset.x as i32) as f32,
                    (y as i32 * tile_size.y as i32 - offset.y as i32) as f32,
                ),
                (tile_size.x as f32, tile_size.y as f32),
            ]);
        }
    }
}

/// Renders field of view as shadows
pub fn render_fov_shadows(
    pass: &mut BatchPass<'_>,
    tiled: &tiled::Map,
    fov: &FovData,
    px_bounds: &Rect2f,
) {
    let tile_size = Vec2u::new(tiled.tile_width, tiled.tile_height);

    let (ys, xs) = self::visible_cells_from_px_bounds(px_bounds, tiled);
    for y in ys[0]..ys[1] {
        for x in xs[0]..xs[1] {
            let alpha = if fov.is_in_view([x as i32, y as i32].into()) {
                let len = (Vec2i::new(x as i32, y as i32) - fov.origin()).len_f32();
                let x = len / fov.radius() as f32;
                ease(x) * 0.5
            } else {
                0.7
            };

            let color = Color::rgba(0, 0, 0, 255).multiply(alpha);

            pass.white_dot()
                .color(color)
                .src_rect_uv([0.0, 0.0, 1.0, 1.0])
                .dest_rect_px([
                    (
                        (x as i32 * tile_size.x as i32 - px_bounds.left_up().x as i32) as f32,
                        (y as i32 * tile_size.y as i32 - px_bounds.left_up().y as i32) as f32,
                    ),
                    (tile_size.x as f32, tile_size.y as f32),
                ]);
        }
    }

    /// x: [0.0, 1.0]
    fn ease(x: f32) -> f32 {
        if x < 0.5 {
            4.0 * x * x * x
        } else {
            1.0 - (-2.0 * x as f32 + 2.0).powf(3.0) / 2.0
        }
    }
}

// --------------------------------------------------------------------------------
// Higher level

/// Renders a tiled map in a bounds in world coordinates, considering fog of war
pub fn render_tiled_consider_fow(
    dcx: &mut DrawContext,
    tiled: &tiled::Map,
    idmap: &GidTextureMap,
    px_bounds: &Rect2f,
    fow: &FowData,
) {
    let grid_bounds = self::grid_bounds_from_pixel_bounds(tiled, &px_bounds);
    let (ys, xs) = self::visible_cells_from_grid_bounds(&grid_bounds);

    let mut pass = dcx.batch();
    for layer in tiled.layers.iter().filter(|l| l.visible) {
        render_tiled_layer_consider_fow(
            &mut pass,
            tiled,
            layer,
            idmap,
            px_bounds.left_up(),
            ys,
            xs,
            fow,
        );
    }
}

// TODO: refactor
fn render_tiled_layer_consider_fow(
    pass: &mut BatchPass<'_>,
    tiled: &tiled::Map,
    layer: &tiled::Layer,
    idmap: &GidTextureMap,
    offset: Vec2f,
    ys: [u32; 2],
    xs: [u32; 2],
    fow: &FowData,
) {
    let tile_size = Vec2u::new(tiled.tile_width, tiled.tile_height);
    let tiles = match layer.tiles {
        LayerData::Finite(ref f) => f,
        LayerData::Infinite(_) => unimplemented!("tiled map infinite layer"),
    };

    for y in ys[0]..ys[1] {
        for x in xs[0]..xs[1] {
            if fow.is_convered([x as usize, y as usize].into()) {
                continue;
            }

            let tile = tiles[y as usize][x as usize];

            let texture = match idmap.gid_to_tile(tile.gid) {
                Some(t) => t,
                None => continue,
            };

            pass.push(&texture).dest_rect_px([
                (
                    (x as i32 * tile_size.x as i32 - offset.x as i32) as f32,
                    (y as i32 * tile_size.y as i32 - offset.y as i32) as f32,
                ),
                (tile_size.x as f32, tile_size.y as f32),
            ]);
        }
    }
}

// --------------------------------------------------------------------------------
// Debug rendering

/// Renders rectangles to non-blocking cells
pub fn render_grids_on_non_blocking_cells(
    pass: &mut BatchPass<'_>,
    tiled: &tiled::Map,
    blocks: &[bool],
    px_bounds: &Rect2f,
) {
    let grid_size = Vec2u::new(tiled.width, tiled.height);
    let tile_size = Vec2u::new(tiled.tile_width, tiled.tile_height);

    let (ys, xs) = self::visible_cells_from_px_bounds(px_bounds, tiled);
    for y in ys[0]..ys[1] {
        for x in xs[0]..xs[1] {
            let ix = (x + y * grid_size.x) as usize;
            if blocks[ix] {
                return;
            }

            let pos = Vec2f::new(
                (x as i32 * tile_size.x as i32 - px_bounds.left_up().x as i32) as f32,
                (y as i32 * tile_size.y as i32 - px_bounds.left_up().y as i32) as f32,
            );

            pass.rect(
                [pos + Vec2f::new(2.0, 2.0), (28.0, 28.0).into()],
                Color::white().multiply(0.5),
            );
        }
    }
}

// --------------------------------------------------------------------------------
// Internal utilities

/// Returns (ys, xs)
fn visible_cells_from_px_bounds(px_bounds: &Rect2f, tiled: &tiled::Map) -> ([u32; 2], [u32; 2]) {
    let grid_bounds = self::grid_bounds_from_pixel_bounds(tiled, px_bounds);
    self::visible_cells_from_grid_bounds(&grid_bounds)
}

/// Returns (ys, xs)
fn visible_cells_from_grid_bounds(grid_bounds: &Rect2i) -> ([u32; 2], [u32; 2]) {
    (
        [
            grid_bounds.up() as u32,
            grid_bounds.up() as u32 + grid_bounds.h(),
        ],
        [
            grid_bounds.left() as u32,
            grid_bounds.left() as u32 + grid_bounds.w(),
        ],
    )
}

fn grid_bounds_from_pixel_bounds(map: &tiled::Map, bounds: &Rect2f) -> Rect2i {
    let left_up = {
        let mut pos = w2t_floor(bounds.left_up(), map);
        pos.x = cmp::max(pos.x, 0);
        pos.y = cmp::max(pos.y, 0);
        pos
    };

    // right down position of the map + [1, 1]
    let right_down = {
        let mut pos = w2t_round_up(bounds.right_down(), map);
        pos.x = cmp::min(pos.x, map.width as i32);
        pos.y = cmp::min(pos.y, map.height as i32);
        pos
    };

    let size = [
        (right_down.x - left_up.x) as u32,
        (right_down.y - left_up.y) as u32,
    ];

    Rect2i::new(left_up, size)
}
