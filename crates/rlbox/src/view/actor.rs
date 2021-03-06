/*!
Frame-based actor sprite animation
*/

use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use snow2d::{
    asset::{Asset, AssetDeState, AssetKey},
    gfx::{
        draw::*,
        geom2d::*,
        tex::{SpriteData, Texture2dDrop},
        Color,
    },
    utils::ez,
};

use crate::{
    rl::grid2d::*,
    utils::{consts, DoubleSwap},
    view::anim::{FrameAnimPattern, FrameAnimState, LoopMode},
};

/// Generate character walking animation with some heuristic
fn gen_anim_auto(
    tex: &Asset<Texture2dDrop>,
    fps: f32,
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    let size = tex.get().unwrap().sub_tex_size_unscaled();
    if size[0] >= size[1] {
        self::gen_anim_dir8(tex, fps)
    } else {
        self::gen_anim_dir4(tex, fps)
    }
}

/// Generates character walking animation from 3x4 character image
fn gen_anim_dir4(
    tex: &Asset<Texture2dDrop>,
    fps: f32,
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    self::gen_dir_anim_with(
        tex,
        fps,
        &DIR_4_ANIM_PATTERN,
        |ix| {
            let row = ix / 3;
            let col = ix % 3;
            // x, y, w, h
            [col as f32 / 3.0, row as f32 / 4.0, 1.0 / 3.0, 1.0 / 4.0]
        },
        |_sprite| {},
    )
}

/// Generates character walking animation from 6x4 character image
fn gen_anim_dir8(
    tex: &Asset<Texture2dDrop>,
    fps: f32,
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    self::gen_dir_anim_with(
        tex,
        fps,
        &DIR_8_ANIM_PATTERN,
        |ix| {
            let row = ix / 6;
            let col = ix % 6;
            // x, y, w, h
            [col as f32 / 6.0, row as f32 / 4.0, 1.0 / 6.0, 1.0 / 4.0]
        },
        |_sprite| {},
    )
}

/// Maps [`Dir8`] to a diretional animation frame
type DirAnimPattern = [(Dir8, [usize; 3]); 8];

const DIR_4_ANIM_PATTERN: DirAnimPattern = [
    (Dir8::E, [6, 7, 8]),
    (Dir8::W, [3, 4, 5]),
    (Dir8::S, [0, 1, 2]),
    (Dir8::SE, [0, 1, 2]),
    (Dir8::SW, [0, 1, 2]),
    (Dir8::N, [9, 10, 11]),
    (Dir8::NE, [9, 10, 11]),
    (Dir8::NW, [9, 10, 11]),
];

const DIR_8_ANIM_PATTERN: DirAnimPattern = [
    (Dir8::E, [12, 13, 14]),
    (Dir8::W, [6, 7, 8]),
    (Dir8::S, [0, 1, 2]),
    (Dir8::SE, [9, 10, 11]),
    (Dir8::SW, [3, 4, 5]),
    (Dir8::N, [18, 19, 20]),
    (Dir8::NE, [21, 22, 23]),
    (Dir8::NW, [15, 16, 17]),
];

fn gen_dir_anim_with(
    tex: &Asset<Texture2dDrop>,
    fps: f32,
    patterns: &DirAnimPattern,
    gen_uv_rect: impl Fn(usize) -> [f32; 4],
    mut f: impl FnMut(&mut SpriteData),
) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
    patterns
        .iter()
        .map(|(dir, indices)| {
            (
                dir.clone(),
                FrameAnimPattern::new(
                    indices
                        .iter()
                        .map(|ix| {
                            let mut sprite = SpriteData {
                                tex: tex.clone(),
                                uv_rect: gen_uv_rect(*ix),
                                rot: 0.0,
                                origin: [0.5, 0.5],
                                scales: [1.0, 1.0],
                                color: Color::WHITE,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirAnimKind {
    Auto,
    Dir4,
    Dir8,
    // TODO: OneImage,
}

impl DirAnimKind {
    fn gen_anim_patterns(
        &self,
        tex: &Asset<Texture2dDrop>,
        fps: f32,
    ) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
        match self {
            Self::Auto => self::gen_anim_auto(tex, fps),
            Self::Dir4 => self::gen_anim_dir4(tex, fps),
            Self::Dir8 => self::gen_anim_dir8(tex, fps),
        }
    }
}

/// Play facing animation based on this
// FIXME: serde image scales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirAnimDesc {
    pub tex: Asset<Texture2dDrop>,
    pub kind: DirAnimKind,
    pub fps: f32,
}

impl DirAnimDesc {
    pub fn gen_anim_patterns(&self) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
        self.kind.gen_anim_patterns(&self.tex, self.fps)
    }

    pub fn gen_anim_state(&self, dir: Dir8) -> FrameAnimState<Dir8, SpriteData> {
        FrameAnimState::new(self.kind.gen_anim_patterns(&self.tex, self.fps), dir)
    }
}

#[derive(Debug, Clone)]
pub struct WalkAnimDesc {
    pub walk_secs: f32,
    pub walk_ease: ez::Ease,
}

/// An animatable actor image
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "ActorImageSerde")]
#[serde(into = "ActorImageSerde")]
pub struct ActorImage {
    dir_anim_state: FrameAnimState<Dir8, SpriteData>,
    diff: DoubleSwap<ActorSnapshot>,
    dir: ez::Tweened<Dir8>,
    /// Interpolation value for walk animation
    walk_dt: ez::EasedDt,
    /// For easy serde with [`ActorImageSerde`]
    dir_anim_desc: DirAnimDesc,
}

/// Interpolate two snapshots to draw actor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct ActorSnapshot {
    pos: Vec2i,
    dir: Dir8,
}

impl ActorImage {
    pub fn new(
        dir_anim_desc: DirAnimDesc,
        walk_ease_desc: ez::EasedDtDesc,
        pos: Vec2i,
        dir: Dir8,
    ) -> Self {
        let data = ActorSnapshot { pos, dir };

        Self {
            dir_anim_state: dir_anim_desc.gen_anim_state(dir),
            diff: DoubleSwap::new(data, data),
            walk_dt: walk_ease_desc.into(),
            dir: ez::Tweened {
                a: dir,
                b: dir,
                dt: ez::EasedDt::completed(),
            },
            dir_anim_desc,
        }
    }

    /// Sets position and direction without animation
    pub fn warp(&mut self, pos: Vec2i, dir: Dir8) {
        let next_snap = ActorSnapshot { dir, pos };
        self.diff.set_a(next_snap);
        self.diff.set_b(next_snap);
        self.dir_anim_state.set_pattern(dir, true);
    }

    /// Updates the image with (new) actor position and direction
    pub fn update(&mut self, dt: Duration, pos: Vec2i, dir: Dir8) {
        let (dir_diff, pos_diff) = (dir != self.diff.a().dir, pos != self.diff.a().pos);

        if dir_diff {
            if pos_diff {
                // rotate instantly
                self.dir = ez::Tweened {
                    a: self.dir.a,
                    b: dir,
                    dt: ez::EasedDt::completed(),
                };
            } else {
                // NOTE: it always animate with rotation
                self.dir = ez::tween_dirs(self.diff.a().dir, dir, consts::CHANGE_DIR_TIME);
            }
        }

        // update direction of the animation
        self.dir.tick(dt);
        self.dir_anim_state.set_pattern(self.dir.get(), false);

        // update interpolation value for walk animation
        if pos_diff {
            self.walk_dt.reset();
        }
        self.walk_dt.tick(dt);

        if pos_diff || dir_diff {
            self.diff.swap();
            let next_snap = ActorSnapshot { dir, pos };
            self.diff.set_a(next_snap);
        }

        // update animation frame
        self.dir_anim_state.tick(dt);
    }

    /// Position in world coordinates, used for like camera
    ///
    /// Align the center of the sprite to the center of the cell.
    pub fn pos_world_centered(&self, tiled: &tiled::Map) -> Vec2f {
        let pos_prev = self.align_center(self.diff.b().pos, tiled);
        let pos_curr = self.align_center(self.diff.a().pos, tiled);

        let mut pos = pos_prev * (1.0 - self.walk_dt.get()) + pos_curr * self.walk_dt.get();
        pos.floor_mut();
        pos
    }

    /// Align the bottom-center of an actor to the bottom-center of a cell
    fn align_center(&self, pos: Vec2i, tiled: &tiled::Map) -> Vec2f {
        crate::render::tiled::t2w_center(pos, &tiled)
    }

    /// Position in world coordinates, used for like rendering actors
    pub fn render_pos_world(&self, tiled: &tiled::Map) -> Vec2f {
        let pos_prev = self.align_render(self.diff.b().pos, tiled);
        let pos_curr = self.align_render(self.diff.a().pos, tiled);

        let mut pos = pos_prev * (1.0 - self.walk_dt.get()) + pos_curr * self.walk_dt.get();
        pos.floor_mut();
        pos
    }

    /// Align the center of the sprite to the bottom-center of the cell
    fn align_render(&self, pos: Vec2i, tiled: &tiled::Map) -> Vec2f {
        let mut pos = crate::render::tiled::t2w_center(pos, &tiled);
        pos.y += tiled.tile_height as f32 / 2.0;
        pos.y -= self.sprite().sub_tex_size_unscaled()[1] / 2.0;
        pos
    }

    /// Sprite for current frame
    pub fn sprite(&self) -> &SpriteData {
        self.dir_anim_state.current_frame()
    }

    /// Used to modify frame animation sprites after loading
    pub fn frames_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut SpriteData> {
        self.dir_anim_state.frames_mut()
    }
}

/// Serde representation of [`ActorImage`]
///
/// It doesn't retrive position and direction, so `warp` the image after deserializatin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorImageSerde {
    dir_anim_desc: DirAnimDesc,
    // TODO: maybe use default?
    walk_anim_desc: ez::EasedDtDesc,
}

impl From<ActorImage> for ActorImageSerde {
    fn from(img: ActorImage) -> Self {
        Self {
            dir_anim_desc: img.dir_anim_desc,
            walk_anim_desc: img.walk_dt.to_desc(),
            // pos: img.diff.a().pos,
            // dir: img.diff.a().dir,
        }
    }
}

impl From<ActorImageSerde> for ActorImage {
    fn from(s: ActorImageSerde) -> ActorImage {
        ActorImage::new(s.dir_anim_desc, s.walk_anim_desc, Vec2i::default(), Dir8::S)
    }
}
