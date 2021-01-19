//! The game world, internals and the GUI

pub mod actor;
pub mod render;
pub mod vi;

use {
    rokol::{
        fons::{Align, FontConfig},
        gfx as rg,
    },
    std::time::{Duration, Instant},
};

use snow2d::{
    asset::AssetCacheAny,
    audio::{asset::MusicPlayer, Audio},
    gfx::{Color, Snow2d},
    input::Input,
};

use rlbox::{
    rl::{
        fov::{FovData, OpacityMap},
        fow::FowData,
        grid2d::*,
        rlmap::TiledRlMap,
    },
    utils::{ez, Double},
};

use self::{actor::*, vi::VInput};

/// Powers the game [`World`]
#[derive(Debug)]
pub struct WorldContext {
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
    //
    pub vi: VInput,
}

impl WorldContext {
    pub fn new(title: String) -> Self {
        let mut snow = unsafe { Snow2d::new() };

        // store default font
        let font_cfg = FontConfig {
            font: {
                // FIXME: font path
                let font = include_bytes!("../../assets_embeded/mplus-1p-regular.ttf");
                let ix = snow
                    .fontbook
                    .stash()
                    .add_font_mem("mplus-1p-regular", font)
                    .unwrap();
                snow.fontbook.stash().set_align(Align::TOP | Align::LEFT);
                ix
            },
            fontsize: crate::consts::DEFAULT_FONT_SIZE,
            line_spacing: crate::consts::DEFAULT_LINE_SPACE,
        };
        snow.fontbook.apply_cfg(&font_cfg);

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
            vi: VInput::new(),
        }
    }

    pub fn event(&mut self, ev: &rokol::app::Event) {
        self.input.event(ev);
    }

    pub fn pre_update(&mut self) {
        self.frame_count += 1;

        // FIXME: use real dt
        self.dt = std::time::Duration::from_nanos(1_000_000_000 / 60);

        self.vi.update(&self.input, self.dt);
    }

    pub fn post_update(&mut self) {
        self.rdr.post_update();
    }

    pub fn render(&mut self) {
        // debug render?
    }

    pub fn on_end_frame(&mut self) {
        self.input.on_end_frame();
    }
}

/// The rougelike game world
///
/// Turn-based game state should be outside of this struct.
#[derive(Debug)]
pub struct World {
    pub map: TiledRlMap,
    pub shadow: Shadow,
    pub entities: Vec<Actor>,
}

/// Lifecycle
impl World {
    pub fn update(&mut self, wcx: &mut WorldContext) {
        for e in &mut self.entities {
            e.img.update(wcx.dt, e.pos, e.dir);
        }
    }
}

/// API
impl World {
    pub fn player(&self) -> &Actor {
        &self.entities[0]
    }

    pub fn player_mut(&mut self) -> &mut Actor {
        &mut self.entities[0]
    }

    pub fn is_blocked(&mut self, pos: Vec2i) -> bool {
        if self.map.rlmap.is_blocked(pos) {
            return true;
        }

        for e in &self.entities {
            if e.pos == pos {
                return true;
            }
        }

        false
    }
}

/// Shadow data for visualization
#[derive(Debug)]
pub struct Shadow {
    /// Field of view
    pub fov: Double<FovData>,
    /// Fog of war (shadow on map)
    pub fow: Double<FowData>,
    pub dt: ez::EasedDt,
    pub is_dirty: bool,
}

impl Shadow {
    pub fn new(radius: [u32; 2], map_size: [usize; 2], anim_secs: f32, ease: ez::Ease) -> Self {
        Self {
            fov: Double {
                a: FovData::new(radius[0], radius[1]),
                b: FovData::new(radius[0], radius[1]),
            },
            fow: Double {
                a: FowData::new(map_size),
                b: FowData::new(map_size),
            },
            dt: ez::EasedDt::new(anim_secs, ease),
            is_dirty: false,
        }
    }

    pub fn make_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn calculate(&mut self, origin: Vec2i, map: &impl OpacityMap) {
        // FoV is always cleared so we just swap them
        self.fov.swap();

        // FoW is continued from the previous state, so we'll copy it
        self.fow.b = self.fow.a.clone();

        self.dt.reset();

        rlbox::rl::fow::calculate_fov_fow(&mut self.fov.a, &mut self.fow.a, None, origin, map);
    }

    /// Call it every frame to animate FoV
    pub fn post_update(&mut self, dt: Duration, map: &impl OpacityMap, player: &Actor) {
        if self.is_dirty {
            self.calculate(player.pos, map);
            self.is_dirty = false;
        }

        self.dt.tick(dt);
    }
}
