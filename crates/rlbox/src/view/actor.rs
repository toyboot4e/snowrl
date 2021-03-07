/*!
Frame-based actor sprite animation
*/

use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use snow2d::{
    asset::Asset,
    gfx::{
        draw::*,
        geom2d::*,
        tex::{SpriteData, Texture2dDrop},
        Color,
    },
    utils::{
        ez,
        type_object::{TypeObject, TypeObjectId, TypeObjectStorage},
    },
};

use crate::{
    rl::grid2d::*,
    utils::DoubleSwap,
    view::anim::{FrameAnimPattern, FrameAnimState, LoopMode},
};

/// Default actor image FPS
pub const ACTOR_FPS: f32 = 4.0;

/// Default actor walk duration
pub const ACTOR_WALK_TIME: f32 = 8.0 / 60.0;

/// Duration in seconds to change direction in 45 degrees
pub const CHANGE_DIR_TIME: f32 = 2.0 / 60.0;

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

/// After deserialization, we have to
///
/// 1. Call [`ActorImage::warp`]
/// 2. Change speed properties of [`ActorImage`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorImageDesc {
    pub tex: Asset<Texture2dDrop>,
    pub kind: DirAnimKind,
    // TODO: scales
    // TODO: offset
}

impl TypeObject for ActorImageDesc {}

impl ActorImageDesc {
    pub fn gen_anim_patterns(&self) -> HashMap<Dir8, FrameAnimPattern<SpriteData>> {
        self.kind.gen_anim_patterns(&self.tex, self::ACTOR_FPS)
    }

    pub fn gen_anim_state(&self, dir: Dir8) -> FrameAnimState<Dir8, SpriteData> {
        FrameAnimState::new(self.kind.gen_anim_patterns(&self.tex, self::ACTOR_FPS), dir)
    }
}

/// Part of internal actor data neededfor visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct ActorState {
    pos: Vec2i,
    dir: Dir8,
}

/// An animatable actor image
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "ActorImageSerde")]
#[serde(into = "ActorImageSerde")]
pub struct ActorImage {
    dir_anim_state: FrameAnimState<Dir8, SpriteData>,
    state_diff: DoubleSwap<ActorState>,
    dir_tweem: ez::Tweened<Dir8>,
    /// Interpolation value for walk animation
    walk_dt: ez::EasedDt,
    /// For deserialization
    serde_repr: ActorImageSerde,
}

impl ActorImage {
    pub fn from_desc(
        desc: &ActorImageDesc,
        walk_dt: ez::EasedDtDesc,
        pos: Vec2i,
        dir: Dir8,
    ) -> Self {
        let data = ActorState { pos, dir };

        Self {
            dir_anim_state: desc.gen_anim_state(dir),
            state_diff: DoubleSwap::new(data, data),
            walk_dt: walk_dt.into(),
            dir_tweem: ez::Tweened {
                a: dir,
                b: dir,
                dt: ez::EasedDt::completed(),
            },
            serde_repr: ActorImageSerde::Embedded(desc.clone()),
        }
    }

    /// Be sure to call [`ActorImage::warp`] and set speed properties after creation
    pub fn from_desc_default(desc: &ActorImageDesc) -> Self {
        Self::from_desc(
            desc,
            ez::EasedDtDesc {
                target: self::ACTOR_WALK_TIME,
                ease: ez::Ease::Linear,
            },
            Vec2i::default(),
            Dir8::S,
        )
    }

    /// Create [`ActorImage`] from `serde` representaiton
    // FIXME: implement it with trait
    pub fn from_serde_repr_default(repr: &ActorImageSerde) -> Self {
        match repr {
            ActorImageSerde::Reference(id) => {
                log::trace!("reference");
                let storage = TypeObjectStorage::get_map::<ActorImageDesc>().unwrap();
                let desc = storage.get(id).unwrap_or_else(|| {
                    panic!(
                        "Unable to find type object with ID `{:?}` of type `{}`",
                        id,
                        std::any::type_name::<ActorImageDesc>()
                    )
                });
                log::trace!("LOAD DESC");
                let mut img = Self::from_desc_default(&desc);
                img.serde_repr = repr.clone();
                img
            }
            ActorImageSerde::Embedded(desc) => Self::from_desc_default(desc),
        }
    }
}

/// Lifecycle
impl ActorImage {
    /// Updates the image with (new) actor position and direction
    pub fn update(&mut self, dt: Duration, pos: Vec2i, dir: Dir8) {
        let (dir_diff, pos_diff) = (
            dir != self.state_diff.a().dir,
            pos != self.state_diff.a().pos,
        );

        if dir_diff {
            if pos_diff {
                // rotate instantly
                self.dir_tweem = ez::Tweened {
                    a: self.dir_tweem.a,
                    b: dir,
                    dt: ez::EasedDt::completed(),
                };
            } else {
                // NOTE: it always animate with rotation
                self.dir_tweem =
                    ez::tween_dirs(self.state_diff.a().dir, dir, self::CHANGE_DIR_TIME);
            }
        }

        // update direction of the animation
        self.dir_tweem.tick(dt);
        self.dir_anim_state.set_pattern(self.dir_tweem.get(), false);

        // update interpolation value for walk animation
        if pos_diff {
            self.walk_dt.reset();
        }
        self.walk_dt.tick(dt);

        if pos_diff || dir_diff {
            self.state_diff.swap();
            let next_snap = ActorState { dir, pos };
            self.state_diff.set_a(next_snap);
        }

        // update animation frame
        self.dir_anim_state.tick(dt);
    }
}

impl ActorImage {
    /// Sets position and direction without animation
    pub fn warp(&mut self, pos: Vec2i, dir: Dir8) {
        let next_snap = ActorState { dir, pos };
        self.state_diff.set_a(next_snap);
        self.state_diff.set_b(next_snap);
        self.dir_anim_state.set_pattern(dir, true);
    }

    /// Position in world coordinates, used for like camera
    ///
    /// Align the center of the sprite to the center of the cell.
    pub fn pos_world_centered(&self, tiled: &tiled::Map) -> Vec2f {
        let pos_prev = self.align_center(self.state_diff.b().pos, tiled);
        let pos_curr = self.align_center(self.state_diff.a().pos, tiled);

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
        let pos_prev = self.align_render(self.state_diff.b().pos, tiled);
        let pos_curr = self.align_render(self.state_diff.a().pos, tiled);

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

/// `serde` representation of [`ActorImage`]: embedded or external definition of [`ActorImageDesc`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorImageSerde {
    /// Retrieve [`ActorImageDesc`] from static type object storage
    Reference(TypeObjectId<ActorImageDesc>),
    /// Embed [`ActorImageDesc`]
    Embedded(ActorImageDesc),
}

impl From<ActorImage> for ActorImageSerde {
    fn from(img: ActorImage) -> Self {
        img.serde_repr
    }
}

impl From<ActorImageSerde> for ActorImage {
    fn from(repr: ActorImageSerde) -> ActorImage {
        log::trace!("from ActorImageSerde");
        ActorImage::from_serde_repr_default(&repr)
    }
}
