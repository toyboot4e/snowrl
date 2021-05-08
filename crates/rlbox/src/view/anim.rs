/*!
Frame-based animation states
*/

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

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

    /// Returns (duration, state) after completion handling
    fn on_tick(&mut self, dt: Duration) -> (Duration, LoopState) {
        let loop_duration = self.loop_duration();
        if dt < loop_duration {
            return (dt, LoopState::Running);
        }

        // on end
        match self.loop_mode {
            // finish
            LoopMode::Once | LoopMode::ClampForever => (loop_duration, LoopState::Stopped),
            // loop
            LoopMode::Loop | LoopMode::PingPong | LoopMode::PingPongOnce => {
                (dt - loop_duration, LoopState::Running)
            }
        }
    }

    /// Index of current animation
    pub fn frame_ix(&self, dt: Duration) -> usize {
        let ms_per_frame = 1000.0 * 1.0 / self.fps;
        let ms_dt = dt.as_millis();
        let frame = (ms_dt / ms_per_frame as u128) as usize;

        let len = self.frames.len();
        match self.loop_mode {
            // ping pong loop
            //
            // [A][B][C][D][C][B][A]..
            //  0  1  2  3  4  5  6 :: frame
            //  0  1  2  3  2  1  0 :: 2 * (len - 1) - frame
            //  where the last frame should be omitted so that it's not duplicated
            LoopMode::PingPong | LoopMode::PingPongOnce if frame >= len => 2 * (len - 1) - frame,
            // not ping pong
            _ => frame,
        }
    }

    /// Current animation frame
    pub fn frame(&self, dt: Duration) -> &F {
        &self.frames[self.frame_ix(dt)]
    }

    fn loop_duration(&self) -> Duration {
        let sec = 1.0 / self.fps * self.n_loop_frames() as f32;
        let ms = (1000.0 * sec) as u64;
        Duration::from_millis(ms)
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

/// Animation state for a single pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimState<F> {
    pattern: AnimPattern<F>,
    // states
    accum: Duration,
    state: LoopState,
}

impl<F> AnimState<F> {
    pub fn new(pattern: AnimPattern<F>) -> Self {
        Self {
            pattern,
            accum: Duration::new(0, 0),
            state: LoopState::Running,
        }
    }

    /// Lifecycle
    pub fn tick(&mut self, dt: Duration) {
        if matches!(self.state, LoopState::Stopped) {
            return;
        }

        self.accum += dt;
        let (next_duration, next_state) = self.pattern.on_tick(self.accum);
        self.accum = next_duration;
        self.state = next_state;
    }

    pub fn pattern(&self) -> &AnimPattern<F> {
        &self.pattern
    }

    pub fn current_frame(&self) -> &F {
        self.pattern.frame(self.accum)
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
