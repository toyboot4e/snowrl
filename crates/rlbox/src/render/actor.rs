//! Frame-based actor sprite animation

use std::collections::HashMap;

use snow2d::gfx::tex::{SharedSubTexture2d, SharedTexture2d, SpriteData};

use crate::{
    render::anim::{FrameAnimPattern, LoopMode},
    rl::grid2d::Dir8,
};

/// Generates character walking animation from 4x3 character image
pub fn gen_anim4(
    texture: &SharedTexture2d,
    fps: f32,
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    self::gen_anim4_with(texture, fps, |_sprite| {})
}

/// Generates character walking animation from 4x3 character image, letting user modify the sprite
pub fn gen_anim4_with(
    texture: &SharedTexture2d,
    fps: f32,
    mut f: impl FnMut(&mut SpriteData),
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    [
        (Dir8::E, [6, 7, 8]),
        (Dir8::W, [3, 4, 5]),
        (Dir8::S, [0, 1, 2]),
        (Dir8::SE, [0, 1, 2]),
        (Dir8::SW, [0, 1, 2]),
        (Dir8::N, [9, 10, 11]),
        (Dir8::NE, [9, 10, 11]),
        (Dir8::NW, [9, 10, 11]),
    ]
    .iter()
    .map(|(dir, ixs)| {
        (
            dir.clone(),
            FrameAnimPattern::new(
                ixs.iter()
                    .map(|ix| {
                        let row = ix / 3;
                        let col = ix % 3;
                        let uv_pos = [col as f32 / 3.0, row as f32 / 4.0];
                        let uv_size = [1.0 / 3.0, 1.0 / 4.0];

                        let mut sprite = SpriteData {
                            sub_tex: SharedSubTexture2d {
                                shared: texture.clone(),
                                uv_rect: [uv_pos[0], uv_pos[1], uv_size[0], uv_size[1]],
                            },
                            rot: 0.0,
                            origin: [0.5, 0.5],
                            scale: [1.0, 1.0],
                        };

                        f(&mut sprite);

                        sprite
                    })
                    .collect::<Vec<_>>(),
                fps,
                LoopMode::PingPong,
            ),
        )
    })
    .collect()
}
