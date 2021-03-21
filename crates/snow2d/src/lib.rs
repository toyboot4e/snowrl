/*!
`snow2d` ❄️ framework
*/

pub extern crate rokol;

pub mod asset;
pub mod audio;
pub mod gfx;
pub mod input;

pub mod ui;
pub mod utils;

use std::time::{Duration, Instant};

use rokol::gfx as rg;

use crate::{
    asset::AssetCacheAny,
    audio::asset::MusicPlayer,
    audio::Audio,
    gfx::{Color, Snow2d, WindowState},
    input::Input,
};

/// Set of generic game contexts
#[derive(Debug)]
pub struct Ice {
    /// Clears target (frame buffer) with cornflower blue color
    pa_blue: rg::PassAction,
    /// 2D renderer
    pub snow: Snow2d,
    /// Audio context
    pub audio: Audio,
    pub music_player: MusicPlayer,
    /// Asset cache for any type
    pub assets: AssetCacheAny,
    pub input: Input,
    /// Delta time from last frame
    dt: Duration,
    frame_count: u64,
    /// When the game started
    pub start_time: Instant,
}

impl Ice {
    pub fn new(snow: Snow2d) -> Self {
        // TODO: don't unwrap
        let audio = unsafe { Audio::create().unwrap() };

        Self {
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            snow,
            audio: audio.clone(),
            music_player: MusicPlayer::new(audio.clone()),
            assets: AssetCacheAny::new(),
            input: Input::new(),
            dt: Duration::new(0, 0),
            frame_count: 0,
            start_time: Instant::now(),
        }
    }

    pub fn dt(&self) -> Duration {
        self.dt
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

/// Lifecycle
impl Ice {
    /// Updates input state
    pub fn event(&mut self, ev: &sdl2::event::Event) {
        self.input.event(ev);
    }

    /// Updates frame counter
    pub fn pre_update(&mut self, dt: std::time::Duration) {
        self.frame_count += 1;
        self.dt = dt;
    }

    /// Updates font texture
    pub fn pre_render(&mut self, window: WindowState) {
        self.snow.pre_render(window);
    }

    /// Debug render?
    pub fn render(&mut self) {
        // debug render?
    }

    /// Updates asset reference counts  and swaps input data buffers
    pub fn on_end_frame(&mut self) {
        self.assets.free_unused();
        self.input.on_end_frame();
    }
}
