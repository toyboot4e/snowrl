/*!
Tiled map rendering
*/

use {
    snow2d::gfx::{draw::*, geom2d::*, Color},
    std::cmp,
    tiled::LayerData,
};

use crate::{
    rl::{
        grid2d::{Rect2i, Vec2i, Vec2u},
        shadow::*,
    },
    view::map::*,
};

/// World coordinates to tile coordinates flooring remaning pixels in a cell
pub fn w2t_floor(w: impl Into<Vec2f>, tiled: &tiled::Map) -> Vec2i {
    let w = w.into();
    let x = w.x as u32 / tiled.tile_width;
    let y = w.y as u32 / tiled.tile_height;
    Vec2i::new(x as i32, y as i32)
}

/// World coordinates to tile coordinates rounding up remaning pixels in a cell
pub fn w2t_round_up(w: impl Into<Vec2f>, tiled: &tiled::Map) -> Vec2i {
    let w = w.into();
    let x = (w.x as u32 + tiled.tile_width - 1) / tiled.tile_width;
    let y = (w.y as u32 + tiled.tile_height - 1) / tiled.tile_height;
    Vec2i::new(x as i32, y as i32)
}

/// Tile coordinates to world coordinates (left-up corner)
pub fn t2w(pos: impl Into<Vec2i>, tiled: &tiled::Map) -> Vec2f {
    let pos = pos.into();
    let x = pos.x as f32 * tiled.tile_width as f32;
    let y = pos.y as f32 * tiled.tile_height as f32;
    Vec2f::new(x, y)
}

/// Tile coordinates to world coordinates (center)
pub fn t2w_center(pos: impl Into<Vec2i>, tiled: &tiled::Map) -> Vec2f {
    let pos = pos.into();
    let x = pos.x as f32 * tiled.tile_width as f32 + tiled.tile_width as f32 / 2.0;
    let y = pos.y as f32 * tiled.tile_height as f32 + tiled.tile_height as f32 / 2.0;
    Vec2f::new(x, y)
}

pub fn grid_bounds_from_pixel_bounds(tiled: &tiled::Map, bounds: &Rect2f) -> Rect2i {
    let left_up = {
        // FIXME: w2t_round_up would be enough?
        let mut pos = w2t_floor(bounds.left_up(), tiled);
        pos.x = cmp::max(pos.x, 0);
        pos.y = cmp::max(pos.y, 0);
        pos
    };

    // right down position of the map + [1, 1]
    let right_down = {
        let mut pos = w2t_round_up(bounds.right_down(), tiled);
        pos.x = cmp::min(pos.x, tiled.width as i32);
        pos.y = cmp::min(pos.y, tiled.height as i32);
        pos
    };

    let size = [
        (right_down.x - left_up.x) as u32,
        (right_down.y - left_up.y) as u32,
    ];

    Rect2i::new(left_up, size)
}

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
// --------------------------------------------------------------------------------
// Rendering

/// Renders a tiled map in a bounds in world coordinates
pub fn render_tiled(
    draw: &mut impl DrawApi,
    tiled: &tiled::Map,
    idmap: &GidTextureMap,
    px_bounds: impl Into<Rect2f>,
    layer_range: impl std::ops::RangeBounds<i32>,
) {
    let px_bounds: Rect2f = px_bounds.into();
    let grid_bounds = self::grid_bounds_from_pixel_bounds(tiled, &px_bounds);
    let (ys, xs) = self::visible_cells_from_grid_bounds(&grid_bounds);

    for layer in tiled.layers.iter().filter(|l| l.visible) {
        // [0-9]+
        let int_name = layer.name.chars().take_while(|p| p.is_digit(10));
        let number = match int_name.collect::<String>().parse::<i32>() {
            Ok(num) => num,
            Err(_err) => continue,
        };
        if layer_range.contains(&number) {
            render_tiled_layer(draw, tiled, layer, idmap, ys, xs);
        }
    }
}

#[inline]
pub fn render_tiled_layer(
    draw: &mut impl DrawApi,
    tiled: &tiled::Map,
    layer: &tiled::Layer,
    idmap: &GidTextureMap,
    ys: [u32; 2],
    xs: [u32; 2],
) {
    let tile_size = Vec2u::new(tiled.tile_width, tiled.tile_height);
    let tiles = match layer.tiles {
        LayerData::Finite(ref f) => f,
        LayerData::Infinite(_) => unimplemented!("tiled map infinite layer"),
    };

    let size = Vec2f::new(tile_size.x as f32, tile_size.y as f32);
    for y in ys[0]..ys[1] {
        for x in xs[0]..xs[1] {
            let tile = tiles[y as usize][x as usize];

            let texture = match idmap.gid_to_tile(tile.gid) {
                Some(t) => t,
                None => continue,
            };

            draw.sprite(&texture).dst_rect_px((
                [
                    (x as i32 * tile_size.x as i32) as f32,
                    (y as i32 * tile_size.y as i32) as f32,
                ],
                size,
            ));
        }
    }
}

/// Renders FoV
pub fn render_fov(draw: &mut impl DrawApi, tiled: &tiled::Map, bounds: &Rect2f, fov: &FovData) {
    let tile_size = Vec2u::new(tiled.tile_width, tiled.tile_height);

    let (ys, xs) = self::visible_cells_from_px_bounds(bounds, tiled);
    for y in ys[0]..ys[1] {
        for x in xs[0]..xs[1] {
            let alpha = self::shadow_alpha_from_fov([x, y], fov);
            self::render_shadow_cell(draw, alpha, [x, y], tile_size);
        }
    }
}

/// Renders FoV blending two (for animation)
pub fn render_fov_blend(
    draw: &mut impl DrawApi,
    tiled: &tiled::Map,
    bounds: &Rect2f,
    fov_new: &FovData,
    fov_old: &FovData,
    blend_factor_new: f32,
) {
    let tile_size = Vec2u::new(tiled.tile_width, tiled.tile_height);

    let (ys, xs) = self::visible_cells_from_px_bounds(bounds, tiled);
    for y in ys[0]..ys[1] {
        for x in xs[0]..xs[1] {
            let alpha = {
                let alpha_new = self::shadow_alpha_from_fov([x, y], fov_new);
                let alpha_old = self::shadow_alpha_from_fov([x, y], fov_old);
                alpha_new * blend_factor_new + alpha_old * (1.0 - blend_factor_new)
            };

            self::render_shadow_cell(draw, alpha, [x, y], tile_size);
        }
    }
}

/// Renders FoV and FoW blending two (for animation)
pub fn render_fov_fow_blend(
    draw: &mut impl DrawApi,
    tiled: &tiled::Map,
    px_bounds: &Rect2f,
    fov_new: &FovData,
    fov_old: &FovData,
    blend_factor_new: f32,
    fow_old: &FowData,
    fow_new: &FowData,
) {
    let tile_size = Vec2u::new(tiled.tile_width, tiled.tile_height);

    let (ys, xs) = self::visible_cells_from_px_bounds(px_bounds, tiled);
    for y in ys[0]..ys[1] {
        for x in xs[0]..xs[1] {
            let alpha = {
                let alpha_new = self::shadow_alpha_for_fov_fow([x, y], fov_new, fow_new);
                let alpha_old = self::shadow_alpha_for_fov_fow([x, y], fov_old, fow_old);

                alpha_new * blend_factor_new + alpha_old * (1.0 - blend_factor_new)
            };

            self::render_shadow_cell(draw, alpha, [x, y], tile_size);
        }
    }
}

#[inline]
fn shadow_alpha_from_fov(pos: [u32; 2], fov: &FovData) -> f32 {
    let pos = Vec2i::new(pos[0] as i32, pos[1] as i32);

    return if fov.is_in_view(pos.into()) {
        let len = (pos - fov.origin()).len_f32();
        let x = len / fov.radius() as f32;
        0.60 * ease_shadow_alpha(x)
    } else {
        0.80
    };

    /// x: [0.0, 1.0]
    /// FIXME: use better easing function for FoV
    fn ease_shadow_alpha(x: f32) -> f32 {
        if x < 0.5 {
            4.0 * x * x * x
        } else {
            1.0 - (-2.0 * x as f32 + 2.0).powf(3.0) / 2.0
        }
    }
}

#[inline]
fn shadow_alpha_for_fov_fow(pos: [u32; 2], fov: &FovData, fow: &FowData) -> f32 {
    let pos = Vec2i::new(pos[0] as i32, pos[1] as i32);

    return if fov.is_in_view(pos.into()) {
        let len = (pos - fov.origin()).len_f32();
        let x = len / fov.radius() as f32;

        0.60 * ease_shadow_alpha(x)
    } else if fow.is_visible([pos.x as usize, pos.y as usize]) {
        0.80
    } else {
        1.00 // TODO: change FoW alpha
    };

    /// x: [0.0, 1.0]
    fn ease_shadow_alpha(x: f32) -> f32 {
        if x < 0.5 {
            4.0 * x * x * x
        } else {
            1.0 - (-2.0 * x as f32 + 2.0).powf(3.0) / 2.0
        }
    }
}

#[inline]
fn render_shadow_cell(draw: &mut impl DrawApi, alpha: f32, pos: [u32; 2], tile_size: Vec2u) {
    let alpha_u8 = (255 as f32 * alpha) as u8;

    draw.white_dot()
        .color(Color::rgba(0, 0, 0, alpha_u8))
        .dst_rect_px([
            (
                (pos[0] as i32 * tile_size.x as i32) as f32,
                (pos[1] as i32 * tile_size.y as i32) as f32,
            ),
            (tile_size.x as f32, tile_size.y as f32),
        ]);
}

// --------------------------------------------------------------------------------
// Debug rendering

/// Renders rectangles to non-blocking cells (TODO: consider FoV)
pub fn mark_non_blocking_cells(
    draw: &mut impl DrawApi,
    tiled: &tiled::Map,
    blocks: &[bool],
    px_bounds: &Rect2f,
) {
    let grid_size = Vec2u::new(tiled.width, tiled.height);
    let tile_size = Vec2u::new(tiled.tile_width, tiled.tile_height);
    let rect_size = Vec2f::new(tile_size.x as f32 - 4.0, tile_size.y as f32 - 4.0);

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

            draw.rect(
                [pos + Vec2f::new(2.0, 2.0), rect_size],
                Color::WHITE.with_alpha(127),
            );
        }
    }
}
