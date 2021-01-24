//! Magic values (should be removed)

use rlbox::utils::ez;

/// FPS of character graphics animation
pub const ACTOR_FPS: f32 = 4.0;

/// Filed of view radius
pub const FOV_R: u32 = 5;

pub const FOV_EASE: ez::Ease = ez::Ease::Linear;

pub const DEFAULT_FONT_SIZE: f32 = 22.0;
pub const DEFAULT_LINE_SPACE: f32 = 4.0;

pub const CHARS_PER_SEC: f32 = 100.0;
pub const TALK_WIN_ANIM_TIME: f32 = 10.0 / 60.0;
pub const TALK_WIN_EASE: ez::Ease = ez::Ease::ExpOut;

/// Walk duration in seconds
pub const WALK_TIME: f32 = 8.0 / 60.0;

pub const WALK_EASE: ez::Ease = ez::Ease::Linear;

/// Key repeat duration for virtual directional key
pub const REPEAT_FIRST_FRAMES: u64 = 10;

/// Key repeat duration for virtual directional key
pub const REPEAT_MULTI_FRAMES: u64 = 6;

/// [left, top]
pub const TALK_PADS: [f32; 2] = [12.0, 8.0];
