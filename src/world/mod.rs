//! The game world, internals and the GUI

pub mod actor;
pub mod render;
pub mod vi;

use {
    rokol::gfx as rg,
    std::time::{Duration, Instant},
};

use snow2d::{gfx::Color, Snow2d};

use rlbox::rl::{
    fov::{FovData, OpacityMap},
    fow::FowData,
    grid2d::*,
    rlmap::TiledRlMap,
};

use crate::utils::Double;

use self::{actor::*, vi::VInput};

/// Powers the game [`World`]
#[derive(Debug)]
pub struct WorldContext {
    /// 2D renderer
    pub rdr: Snow2d,
    pub soloud: soloud::Soloud,
    /// Clears target (frame buffer) with cornflower blue color
    pa_blue: rg::PassAction,
    pub input: xdl::Input,
    pub vi: VInput,
    /// Delta time from last frame
    pub dt: Duration,
    pub frame_count: u64,
    /// When the game started
    pub start_time: Instant,
}

impl Default for WorldContext {
    fn default() -> Self {
        Self {
            rdr: unsafe { Snow2d::new() },
            // TODO: do not unwrap and make a dummy
            soloud: soloud::Soloud::default().unwrap(),
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            input: xdl::Input::new(),
            vi: VInput::new(),
            dt: Duration::new(0, 0),
            frame_count: 0,
            start_time: Instant::now(),
        }
    }
}

impl WorldContext {
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
    pub entities: Vec<Player>,
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
    pub fn player(&self) -> &Player {
        &self.entities[0]
    }

    pub fn player_mut(&mut self) -> &mut Player {
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

/// Shadow data suitable for visualization
#[derive(Debug)]
pub struct Shadow {
    /// Field of view
    pub fov: Double<FovData>,
    /// Fog of war (shadow on map)
    pub fow: Double<FowData>,
    /// Used to render FoV
    pub blend_factor: f32,
    pub is_dirty: bool,
}

impl Shadow {
    pub fn make_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn calculate(&mut self, origin: Vec2i, map: &impl OpacityMap) {
        // FoV is always cleared so we just swap them
        self.fov.swap();

        // FoW is continued from the previous state, so we'll copy it
        self.fow.b = self.fow.a.clone();

        // `self.blend_factor` is `tick`ed later in this frame
        self.blend_factor = 0.0;

        rlbox::rl::fow::calculate_fov_fow(&mut self.fov.a, &mut self.fow.a, None, origin, map);
    }

    /// Call it every frame to animate FoV
    pub fn post_update(&mut self, dt: Duration, map: &impl OpacityMap, player: &Player) {
        if self.is_dirty {
            self.calculate(player.pos, map);
            self.is_dirty = false;
        }

        self.tick(dt);
    }

    fn tick(&mut self, dt: Duration) {
        self.blend_factor += dt.as_secs_f32() / crate::consts::WALK_TIME;
        if self.blend_factor >= 1.0 {
            self.blend_factor = 1.0;
        }
    }
}
