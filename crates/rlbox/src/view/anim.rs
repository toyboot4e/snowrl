/*!
Frame-based animation states
*/

use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};
use snow2d::{
    asset::Asset,
    gfx::tex::{SpriteData, Texture2dDrop},
    input::Dir8,
    utils::tyobj::*,
};

/// Option for playing frame-based animation patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoopMode {
    /// ABC ABC ABC ..
    Loop,
    /// ABCCCC..
    ClampForever,
    /// ABC <stop> (pause and set time to 0)
    Once,
    /// ABCB ABCB ABCB ..
    PingPong,
    /// ABCBA <stop> (pause and set time to 0)
    PingPongOnce,
}

/// Running | Paused | Stopped
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoopState {
    Running,
    Paused,
    Stopped,
}

/// Frame-based animation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimPattern<F> {
    frames: Vec<F>,
    /// TODO: use sec per frame?
    fps: f32,
    loop_mode: LoopMode,
}

impl<F> AnimPattern<F> {
    pub fn new(frames: Vec<F>, fps: f32, loop_mode: LoopMode) -> Self {
        Self {
            frames,
            fps,
            loop_mode,
        }
    }

    /// Returns (acccumulated_duration, state) after completion handling
    fn on_tick(&mut self, accum: Duration) -> (Duration, LoopState) {
        let loop_duration = self.loop_duration();

        if accum < loop_duration {
            return (accum, LoopState::Running);
        }

        // on end
        match self.loop_mode {
            // finish
            LoopMode::Once | LoopMode::ClampForever => (loop_duration, LoopState::Stopped),
            // loop
            LoopMode::Loop | LoopMode::PingPong | LoopMode::PingPongOnce => {
                (accum - loop_duration, LoopState::Running)
            }
        }
    }

    /// Index of current animation
    pub fn frame_ix(&self, dt: Duration) -> usize {
        let frame = {
            let ms_per_frame = 1000.0 * 1.0 / self.fps;
            let ms_dt = dt.as_millis();
            (ms_dt / ms_per_frame as u128) as usize
        };

        let len = self.frames.len();

        match self.loop_mode {
            // ping pong loop
            //
            // [A][B][C][D][C][B][A]..
            //  0  1  2  3  4  5  6 :: frame
            //  0  1  2  3  2  1  0 :: 2 * (len - 1) - frame
            //  where the last frame should be omitted so that it's not duplicated
            LoopMode::PingPong | LoopMode::PingPongOnce if frame >= len => 2 * (len - 1) - frame,
            // not ping pong (clamp)
            // FIXME: we should not need the min bounds
            _ => usize::min(frame, len - 1),
        }
    }

    /// Current animation frame
    pub fn frame(&self, dt: Duration) -> &F {
        &self.frames[self.frame_ix(dt)]
    }

    fn loop_duration(&self) -> Duration {
        let secs = 1.0 / self.fps * self.n_loop_frames() as f32;
        Duration::from_secs_f32(secs)
    }

    fn n_loop_frames(&self) -> usize {
        match self.loop_mode {
            // ping pong (omitting the duplicating last frame)
            LoopMode::PingPong | LoopMode::PingPongOnce => self.frames.len() * 2 - 2,
            // not ping pong
            LoopMode::Loop | LoopMode::Once | LoopMode::ClampForever => self.frames.len(),
        }
    }
}

/// Animation state that can switch among registored patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPatternAnimState<K, F>
where
    K: Eq + std::hash::Hash,
{
    // pattern settings
    patterns: HashMap<K, AnimPattern<F>>,
    // states
    cur_key: K,
    accum: Duration,
    state: LoopState,
}

impl<K, F> MultiPatternAnimState<K, F>
where
    K: Eq + std::hash::Hash,
{
    pub fn new(patterns: HashMap<K, AnimPattern<F>>, initial_key: K) -> Self {
        Self {
            patterns,
            cur_key: initial_key,
            accum: Duration::new(0, 0),
            state: LoopState::Running,
        }
    }

    /// Lifecycle
    pub fn tick(&mut self, dt: Duration) {
        if matches!(self.state, LoopState::Stopped) {
            return;
        }

        let pattern = self.patterns.get_mut(&self.cur_key).unwrap();

        self.accum += dt;
        let (next_duration, next_state) = pattern.on_tick(self.accum);
        self.accum = next_duration;
        self.state = next_state;
    }

    /// Current animation pattern, i.e., current animation frames
    pub fn current_pattern(&self) -> Option<&AnimPattern<F>> {
        self.patterns.get(&self.cur_key)
    }

    pub fn current_frame(&self) -> Option<&F> {
        self.current_pattern().map(|p| p.frame(self.accum))
    }

    pub fn set_pattern(&mut self, key: K, reset_accum: bool) {
        if reset_accum {
            self.accum = Duration::new(0, 0);
        }

        self.cur_key = key;
    }

    pub fn patterns_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut F> {
        self.patterns.values_mut().flat_map(|p| p.frames.iter_mut())
    }
}

/// Type object of directional animation performed by entity
#[derive(Debug, Clone, Serialize, Deserialize, TypeObject)]
pub struct DirAnimType {
    tex: Asset<Texture2dDrop>,
    dir: AnimDir,
    /// Frames to wait per div
    wait: u32,
    /// Numbers of \[vertical, horizontal\] division
    div: [usize; 2],
    /// TODO: allow range repr
    frames: Vec<usize>,
    #[serde(default = "DirAnimType::default_anim_scale")]
    scale: [f32; 2],
    #[serde(default = "DirAnimType::default_origin")]
    origin: [f32; 2],
    // TODO: add offset
}

/// If the animation should change angle depending on the actor
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnimDir {
    Fixed(f32),
    Offset(f32),
}

impl DirAnimType {
    fn default_anim_scale() -> [f32; 2] {
        [1.0, 1.0]
    }

    fn default_origin() -> [f32; 2] {
        [0.5, 0.5]
    }

    pub fn to_pattern(&self) -> AnimPattern<SpriteData> {
        let mut s = SpriteData::builder(self.tex.clone());
        let w = 1.0 / self.div[0] as f32;
        let h = 1.0 / self.div[1] as f32;

        let frames = (0..(self.div[0] * self.div[1]))
            .map(|i| {
                let u = (i % self.div[0]) as f32 * w;
                let v = (i / self.div[0]) as f32 * h;

                s.uv_rect([u, v, w, h])
                    .scales(self.scale)
                    .origin(self.origin)
                    .build()
            })
            .collect::<Vec<_>>();

        AnimPattern {
            frames,
            fps: 60.0 / self.wait as f32,
            loop_mode: LoopMode::ClampForever,
        }
    }
}

/// State  of directional animation performed by an entity
#[derive(Debug, Clone, Serialize, Deserialize, SerdeViaTyObj)]
#[via_tyobj(tyobj = "DirAnimType", from_tyobj = "Self::from_type")]
pub struct DirAnimState {
    pattern: AnimPattern<SpriteData>,
    dir: AnimDir,
    // states
    accum: Duration,
    state: LoopState,
}

impl DirAnimState {
    pub fn from_patterns(pattern: AnimPattern<SpriteData>, dir: AnimDir) -> Self {
        Self {
            pattern,
            dir,
            accum: Duration::new(0, 0),
            state: LoopState::Running,
        }
    }

    pub fn from_type(desc: &DirAnimType) -> Self {
        Self::from_patterns(desc.to_pattern(), desc.dir)
    }

    /// Lifecycle
    pub fn tick(&mut self, dt: Duration) {
        if matches!(self.state, LoopState::Stopped) {
            return;
        }

        let (accum, next_state) = self.pattern.on_tick(self.accum + dt);
        self.accum = accum;
        self.state = next_state;
    }

    pub fn pattern(&self) -> &AnimPattern<SpriteData> {
        &self.pattern
    }

    pub fn current_frame(&self) -> &SpriteData {
        self.pattern.frame(self.accum)
    }

    /// TODO: change angle depending on the actor's direction
    pub fn current_frame_with_dir(&self, _dir: Dir8) -> SpriteData {
        match self.dir {
            AnimDir::Fixed(offset) if offset == 0.0 => self.pattern.frame(self.accum).clone(),
            AnimDir::Fixed(_offset) => self.pattern.frame(self.accum).clone(),
            AnimDir::Offset(_offset) => self.pattern.frame(self.accum).clone(),
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.state == LoopState::Stopped
    }

    pub fn state(&self) -> LoopState {
        self.state
    }
}
