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

use rokol::{fons::FontConfig, gfx as rg};

use crate::{
    asset::AssetCacheAny,
    audio::asset::MusicPlayer,
    audio::Audio,
    gfx::{Color, Snow2d},
    input::Input,
};

/// Set of generic game contexts
#[derive(Debug)]
pub struct Ice {
    /// TODO: For debug purpose
    window_title: String,
    /// Clears target (frame buffer) with cornflower blue color
    pa_blue: rg::PassAction,
    /// 2D renderer
    pub rdr: Snow2d,
    /// Default font configuration
    pub font_cfg: FontConfig,
    /// Audio context
    pub audio: Audio,
    pub music_player: MusicPlayer,
    /// Asset cache for any type
    pub assets: AssetCacheAny,
    pub input: Input,
    /// Delta time from last frame
    pub dt: Duration,
    pub frame_count: u64,
    /// When the game started
    pub start_time: Instant,
}

impl Ice {
    pub fn new(title: String, snow: Snow2d, font_cfg: FontConfig) -> Self {
        // TODO: don't unwrap
        let audio = unsafe { Audio::create().unwrap() };

        Self {
            window_title: title,
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            rdr: snow,
            font_cfg,
            audio: audio.clone(),
            music_player: MusicPlayer::new(audio.clone()),
            assets: AssetCacheAny::new(),
            input: Input::new(),
            dt: Duration::new(0, 0),
            frame_count: 0,
            start_time: Instant::now(),
        }
    }
}

/// Lifecycle
impl Ice {
    /// Updates input state
    pub fn event(&mut self, ev: &rokol::app::Event) {
        self.input.event(ev);
    }

    /// Updates frame counter
    pub fn pre_update(&mut self) {
        self.frame_count += 1;
    }

    /// Updates font texture
    pub fn pre_render(&mut self) {
        self.rdr.pre_render();
        // FIXME: use real dt
        self.dt = std::time::Duration::from_nanos(1_000_000_000 / 60);
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
