/*!
Frame-based actor sprite animation

NOTE: actor renderer is NOT in this crate.
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
    ui::{node, Layer, Node},
    utils::{
        ez,
        pool::Handle,
        tyobj::{SerdeRepr, SerdeViaTyObj, TypeObject},
        Inspect,
    },
};

use crate::{
    rl::grid2d::*,
    utils::DoubleSwap,
    view::anim::{AnimPattern, LoopMode, MultiPatternAnimState},
};

/// Default actor image FPS
pub const ACTOR_FPS: f32 = 4.0;

/// Default actor walk duration
pub const ACTOR_WALK_TIME: f32 = 8.0 / 60.0;

/// Duration in seconds to change direction in 45 degrees
pub const CHANGE_DIR_TIME: f32 = 1.0 / 60.0;

/// Generate character walking animation with some heuristic
fn gen_anim_auto(tex: &Asset<Texture2dDrop>, fps: f32) -> HashMap<Dir8, AnimPattern<SpriteData>> {
    let size = tex.get().unwrap().sub_tex_size_unscaled();
    if size[0] >= size[1] {
        self::gen_anim_dir8(tex, fps)
    } else {
        self::gen_anim_dir4(tex, fps)
    }
}

/// Generates character walking animation from 3x4 character image
fn gen_anim_dir4(tex: &Asset<Texture2dDrop>, fps: f32) -> HashMap<Dir8, AnimPattern<SpriteData>> {
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
fn gen_anim_dir8(tex: &Asset<Texture2dDrop>, fps: f32) -> HashMap<Dir8, AnimPattern<SpriteData>> {
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
) -> HashMap<Dir8, AnimPattern<SpriteData>> {
    patterns
        .iter()
        .map(|(dir, indices)| {
            (
                dir.clone(),
                AnimPattern::new(
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Inspect)]
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
    ) -> HashMap<Dir8, AnimPattern<SpriteData>> {
        match self {
            Self::Auto => self::gen_anim_auto(tex, fps),
            Self::Dir4 => self::gen_anim_dir4(tex, fps),
            Self::Dir8 => self::gen_anim_dir8(tex, fps),
        }
    }
}

/// Type object of [`ActorImage`]
///
/// After deserialization, we have to
///
/// 1. Call [`ActorImage::warp`]
/// 2. Set speed properties of [`ActorImage`]
#[derive(Debug, Clone, Serialize, Deserialize, Inspect, TypeObject)]
pub struct ActorImageType {
    pub tex: Asset<Texture2dDrop>,
    pub kind: DirAnimKind,
    // TODO: scales
    // TODO: offset
}

impl ActorImageType {
    pub fn gen_anim_patterns(&self) -> HashMap<Dir8, AnimPattern<SpriteData>> {
        self.kind.gen_anim_patterns(&self.tex, self::ACTOR_FPS)
    }

    pub fn gen_anim_state(&self, dir: Dir8) -> MultiPatternAnimState<Dir8, SpriteData> {
        MultiPatternAnimState::new(self.kind.gen_anim_patterns(&self.tex, self::ACTOR_FPS), dir)
    }
}

/// Part of internal actor data neededfor visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Inspect)]
struct ActorState {
    pos: Vec2i,
    dir: Dir8,
}

/// An animatable actor image
#[derive(Debug, Clone, Serialize, Deserialize, SerdeViaTyObj)]
#[via_tyobj(
    tyobj = "ActorImageType",
    from_tyobj = "Self::from_desc_default",
    repr_field = "serde_repr"
)]
#[serde(from = "SerdeRepr<ActorImageType>")]
#[serde(into = "SerdeRepr<ActorImageType>")]
pub struct ActorImage {
    dir_anim_state: MultiPatternAnimState<Dir8, SpriteData>,
    state_diff: DoubleSwap<ActorState>,
    dir_tween: ez::Tweened<Dir8>,
    /// Interpolation value for walk animation
    walk_dt: ez::EasedDt,
    /// For deserialization
    serde_repr: SerdeRepr<ActorImageType>,
}

impl Inspect for ActorImage {
    fn inspect(&mut self, ui: &imgui::Ui, label: &str) {
        ui.label_text(
            &imgui::im_str!("{}", label),
            &imgui::im_str!("TODO: ActorImage"),
        );
    }
}

impl ActorImage {
    pub fn from_desc(
        desc: &ActorImageType,
        walk_dt: ez::EasedDtDesc,
        pos: Vec2i,
        dir: Dir8,
    ) -> Self {
        let data = ActorState { pos, dir };

        Self {
            dir_anim_state: desc.gen_anim_state(dir),
            state_diff: DoubleSwap::new(data, data),
            walk_dt: walk_dt.into(),
            dir_tween: ez::Tweened {
                a: dir,
                b: dir,
                dt: ez::EasedDt::completed(),
            },
            serde_repr: SerdeRepr::Embedded(desc.clone()),
        }
    }

    pub fn from_desc_default(desc: &ActorImageType) -> Self {
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
}

/// Lifecycle
impl ActorImage {
    /// Moves/animates/tweens the image
    pub fn update(&mut self, dt: Duration, pos: Vec2i, dir: Dir8) {
        let (dir_diff, pos_diff) = (
            dir != self.state_diff.a().dir,
            pos != self.state_diff.a().pos,
        );

        if dir_diff {
            if pos_diff {
                // rotate instantly
                self.dir_tween = ez::Tweened {
                    a: self.dir_tween.a,
                    b: dir,
                    dt: ez::EasedDt::completed(),
                };
            } else {
                // NOTE: it always animates with rotation
                self.dir_tween =
                    ez::tween_dirs(self.state_diff.a().dir, dir, self::CHANGE_DIR_TIME);
            }
        }

        // update direction of the animation
        self.dir_tween.tick(dt);
        self.dir_anim_state.set_pattern(self.dir_tween.get(), false);

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

    /// Sprite for current frame
    pub fn sprite(&self) -> &SpriteData {
        self.dir_anim_state.current_frame().unwrap()
    }
}

/// Coordinates
impl ActorImage {
    /// Position in world coordinates. This is common among various sizes of images, so suitable for
    /// e.g., camera.
    ///
    /// Align the center of the sprite to the center of the cell.
    pub fn pos_world_centered(&self, tiled: &tiled::Map) -> Vec2f {
        let pos_prev = self.align_cell_center(self.state_diff.b().pos, tiled);
        let pos_curr = self.align_cell_center(self.state_diff.a().pos, tiled);
        let mut pos = self.walk_dt.lerp(pos_prev, pos_curr);
        pos.floor_mut();
        pos
    }

    /// Align the bottom-center of an actor to the bottom-center of a cell
    fn align_cell_center(&self, pos: Vec2i, tiled: &tiled::Map) -> Vec2f {
        crate::render::tiled::t2w_center(pos, &tiled)
    }

    /// Base node position in world coordinates
    pub fn base_pos_world(&self, tiled: &tiled::Map) -> Vec2f {
        let pos_prev = Self::align_base(self.state_diff.b().pos, tiled);
        let pos_curr = Self::align_base(self.state_diff.a().pos, tiled);
        let mut pos = self.walk_dt.lerp(pos_prev, pos_curr);
        pos.floor_mut();
        pos
    }

    /// Align the center of the sprite to the bottom-center of the cell
    fn align_base(pos: Vec2i, tiled: &tiled::Map) -> Vec2f {
        let delta = Vec2f::new(0.0, tiled.tile_height as f32 / 2.0);
        crate::render::tiled::t2w_center(pos, &tiled) + delta
    }

    /// Image position in world coordinates
    pub fn img_pos_world(&self, tiled: &tiled::Map) -> Vec2f {
        let pos_prev = self.align_img(self.state_diff.b().pos, tiled);
        let pos_curr = self.align_img(self.state_diff.a().pos, tiled);
        let mut pos = self.walk_dt.lerp(pos_prev, pos_curr);
        pos.floor_mut();
        pos
    }

    /// Align the center of the sprite to the bottom-center of the cell
    fn align_img(&self, pos: Vec2i, tiled: &tiled::Map) -> Vec2f {
        Self::align_base(pos, &tiled) + self.img_offset()
    }

    pub fn img_offset(&self) -> Vec2f {
        Vec2f::new(0.0, -self.sprite().sub_tex_size_unscaled()[1] / 2.0)
    }
}

/// Handle of nodes for an actor in a pool
#[derive(Debug, Clone, Inspect)]
pub struct ActorNodes {
    /// Other nodes are stored as children of this node
    pub base: Handle<Node>,
    pub img: Handle<Node>,
    pub hp: Handle<Node>,
    // /// Non-ordinary view components
    // pub extras: Vec<Handle<Node>>,
}

impl ActorNodes {
    pub fn new(layer: &mut Layer, img_sprite: &SpriteData) -> Self {
        let base = layer.nodes.add(node::Draw::None);

        let h = img_sprite.sub_tex_size_unscaled()[1];
        let mut img = Node::from(img_sprite);
        img.params.pos = Vec2f::new(0.0, -h / 2.0);

        let img = layer.nodes.add_as_child(&base, img);

        // TODO: show HP?
        let hp = layer
            .nodes
            .add_as_child(&base, node::Text::new(format!("")).into());

        Self { base, img, hp }
    }
}
