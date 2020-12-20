//! Frame-based animation states

use std::{collections::HashMap, time::Duration};

/// Option for playing frame-based animation patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoopState {
    Running,
    Paused,
    Stopped,
}

/// Frame-based animation pattern
#[derive(Debug, Clone)]
pub struct FrameAnimPattern<T> {
    pub frames: Vec<T>,
    fps: f32,
    loop_mode: LoopMode,
}

impl<T> FrameAnimPattern<T> {
    pub fn new(frames: Vec<T>, fps: f32, loop_mode: LoopMode) -> Self {
        Self {
            frames,
            fps,
            loop_mode,
        }
    }

    /// Returns (duration, state) after completion handling
    fn on_tick(&mut self, past: Duration) -> (Duration, LoopState) {
        let loop_duration = self.loop_duration();
        if past < loop_duration {
            return (past, LoopState::Running);
        }

        // on end
        match self.loop_mode {
            // finish
            LoopMode::Once | LoopMode::ClampForever => (loop_duration, LoopState::Stopped),
            // loop
            LoopMode::Loop | LoopMode::PingPong | LoopMode::PingPongOnce => {
                (past - loop_duration, LoopState::Running)
            }
        }
    }

    pub fn frame(&self, past: Duration) -> &T {
        &self.frames[self.frame_ix(past)]
    }

    fn frame_ix(&self, past: Duration) -> usize {
        let ms_per_frame = 1000.0 * 1.0 / self.fps;
        let ms_past = past.as_millis();
        let frame = (ms_past / ms_per_frame as u128) as usize;

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

/// Frame-based animation state, composed of paterns
///
/// Animation patterns are selected by keys, which is often `enum`s.
#[derive(Debug, Clone)]
pub struct FrameAnimState<K, T> {
    // pattern settings
    patterns: HashMap<K, FrameAnimPattern<T>>,
    // states
    cur_key: K,
    accum: Duration,
    state: LoopState,
}

impl<K, T> FrameAnimState<K, T> {
    pub fn new(patterns: HashMap<K, FrameAnimPattern<T>>, initial_key: K) -> Self {
        Self {
            patterns,
            cur_key: initial_key,
            accum: Duration::new(0, 0),
            state: LoopState::Running,
        }
    }
}

impl<K: Eq + std::hash::Hash, T> FrameAnimState<K, T> {
    /// Lifecycle
    pub fn tick(&mut self, past: Duration) {
        if matches!(self.state, LoopState::Stopped | LoopState::Stopped) {
            return;
        }

        let pattern = self.patterns.get_mut(&self.cur_key).unwrap();

        self.accum += past;
        let (next_duration, next_state) = pattern.on_tick(self.accum);
        self.accum = next_duration;
        self.state = next_state;
    }

    pub fn current_frame(&mut self) -> &T {
        let pattern = self.patterns.get_mut(&self.cur_key).unwrap();
        pattern.frame(self.accum)
    }

    pub fn current_pattern(&mut self) -> Option<&FrameAnimPattern<T>> {
        self.patterns.get(&self.cur_key)
    }

    pub fn set_pattern(&mut self, key: K, reset_accum: bool) {
        if reset_accum {
            self.accum = Duration::new(0, 0);
        }

        self.cur_key = key;
    }
}
