//! The game world, internals and GUI

pub mod actor;
pub mod render;
pub mod vi;

use {
    rokol::gfx as rg,
    std::{
        path::Path,
        time::{Duration, Instant},
    },
};

use snow2d::{
    asset,
    gfx::{batcher::draw::*, Color},
    PassConfig, Snow2d,
};

use rlbox::rl::{
    self,
    fov::{FovData, FovWrite, OpacityMap},
    fow::FowData,
    grid2d::*,
    rlmap::TiledRlMap,
};

use crate::utils::Double;

use self::{
    actor::*,
    render::{FovRenderer, SnowRenderer},
    vi::VInput,
};

/// Powers the game [`World`]
#[derive(Debug)]
pub struct WorldContext {
    /// 2D renderer
    pub rdr: Snow2d,
    pub soloud: soloud::Soloud,
    /// Clears target (frame buffer) with cornflower blue color
    pa_blue: rg::PassAction,
    pub fov_render: FovRenderer,
    pub snow_render: SnowRenderer,
    pub input: xdl::Input,
    pub vi: VInput,
    pub dt: Duration,
    pub frame_count: u64,
    /// When the game started
    pub start_time: Instant,
}

impl WorldContext {
    pub fn new() -> Self {
        Self {
            rdr: unsafe { Snow2d::new() },
            // TODO: do not unwrap
            soloud: soloud::Soloud::default().unwrap(),
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            fov_render: FovRenderer::default(),
            snow_render: SnowRenderer::default(),
            input: xdl::Input::new(),
            vi: VInput::new(),
            dt: Duration::new(0, 0),
            frame_count: 0,
            start_time: Instant::now(),
        }
    }

    pub fn event(&mut self, ev: &rokol::app::Event) {
        self.input.event(ev);
    }

    pub fn update(&mut self) {
        self.frame_count += 1;
        // FIXME: use real dt
        self.dt = std::time::Duration::from_nanos(1_000_000_000 / 60);

        // input
        self.vi.dir.update(&self.input, self.dt);
    }

    pub fn render(&mut self) {
        // debug render?
    }

    pub fn on_end_frame(&mut self) {
        self.input.on_end_frame();
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
}

impl Shadow {
    pub fn calculate(&mut self, origin: Vec2i, radius: u32, map: &impl OpacityMap) {
        // FoV is always cleared so we just swap them
        self.fov.swap();

        // FoW is continued from the previous state, so we'll copy it
        self.fow.b = self.fow.a.clone();

        // `self.blend_factor` is `tick`ed later in this frame
        self.blend_factor = 0.0;

        rlbox::rl::fow::calculate_fov_fow(
            &mut self.fov.a,
            &mut self.fow.a,
            Some(radius),
            origin,
            map,
        );
    }

    /// Call it every frame to animate FoV
    pub fn update(&mut self, dt: Duration) {
        self.tick(dt);
    }

    fn tick(&mut self, dt: Duration) {
        self.blend_factor += dt.as_secs_f32() / crate::consts::WALK_TIME;
        if self.blend_factor >= 1.0 {
            self.blend_factor = 1.0;
        }
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
    pub fn from_tiled_file(path: &Path) -> anyhow::Result<Self> {
        let map = TiledRlMap::from_tiled_path(path)?;

        let mut shadow = Shadow {
            fov: Double {
                a: FovData::new(crate::consts::FOV_R, 10),
                b: FovData::new(crate::consts::FOV_R, 10),
            },
            fow: Double {
                a: FowData::new(map.rlmap.size),
                b: FowData::new(map.rlmap.size),
            },
            blend_factor: 0.0,
        };

        let mut entities = Vec::with_capacity(20);

        // TODO: use asset loader to make use of cache
        let img = {
            let pos = Vec2i::new(20, 16);
            let dir = Dir8::S;
            ActorImage::from_path(asset::path("ika-chan.png"), pos, dir)?
        };

        entities.push({
            let pos = Vec2i::new(20, 16);
            let dir = Dir8::S;

            let player = Player {
                pos,
                dir,
                img: {
                    let mut img = img.clone();
                    img.force_set(pos, dir);
                    img
                },
            };

            shadow.calculate(player.pos, crate::consts::FOV_R, &map.rlmap);

            player
        });

        entities.push({
            let pos = Vec2i::new(14, 12);
            let dir = Dir8::S;
            Player {
                pos,
                dir,
                img: {
                    let mut img = img.clone();
                    img.force_set(pos, dir);
                    img
                },
            }
        });

        entities.push({
            let pos = Vec2i::new(25, 18);
            let dir = Dir8::S;
            Player {
                pos,
                dir,
                img: {
                    let mut img = img.clone();
                    img.force_set(pos, dir);
                    img
                },
            }
        });

        Ok(Self {
            map,
            shadow,
            entities,
        })
    }

    pub fn event(&mut self, _wcx: &mut WorldContext, _ev: &rokol::app::Event) {}

    pub fn update_images(&mut self, wcx: &mut WorldContext) {
        for e in &mut self.entities {
            e.img.update(wcx.dt, e.pos, e.dir);
        }
    }

    pub fn render(&mut self, wcx: &mut WorldContext) {
        let mut screen = wcx.rdr.screen(PassConfig {
            pa: &wcx.pa_blue,
            tfm: None,
            pip: None,
        });

        self::render::render_tiled(&mut screen, self);
        self.render_actors(&mut screen);

        drop(screen);

        wcx.snow_render.render();

        wcx.fov_render.render_ofs(&mut wcx.rdr, self);
        wcx.fov_render.blend_to_screen(&mut wcx.rdr);

        unsafe {
            // update fontbook GPU texture
            // TODO: it may not work on the first frame
            wcx.rdr.fontbook.update_image();
        }
    }

    fn render_actors(&mut self, draw: &mut impl DrawApi) {
        // TODO: y sort + culling
        for e in &self.entities {
            e.img.render(draw, &self.map.tiled);
        }
    }

    pub fn on_end_frame(&mut self, _wcx: &mut WorldContext) {
        //
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

pub fn update_fov(fov: &mut impl FovWrite, pos: Vec2i, r: u32, map: &impl OpacityMap) {
    rl::fov::refresh(
        fov,
        rl::fov::RefreshParams {
            r,
            origin: pos,
            opa: map,
        },
    );
}
