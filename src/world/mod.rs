//! The game world, internals and GUI

pub mod actor;
pub mod render;
pub mod vi;

use {
    rokol::gfx as rg,
    std::{path::Path, time::Duration},
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

use self::{actor::*, render::FovRenderer, vi::VInput};

/// Powers the game [`World`]
// #[derive(Debug)]
pub struct WorldContext {
    /// 2D renderer
    pub rdr: Snow2d,
    pub soloud: soloud::Soloud,
    /// Clears target (frame buffer) with cornflower blue color
    pa_blue: rg::PassAction,
    pub fov_render: FovRenderer,
    pub input: xdl::Input,
    pub vi: VInput,
    pub dt: Duration,
}

impl WorldContext {
    pub fn new() -> Self {
        Self {
            rdr: unsafe { Snow2d::new() },
            // TODO: do not unwrap
            soloud: soloud::Soloud::default().unwrap(),
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            fov_render: FovRenderer::new(),
            input: xdl::Input::new(),
            vi: VInput::new(),
            dt: Duration::new(0, 0),
        }
    }

    pub fn event(&mut self, ev: &rokol::app::Event) {
        self.input.event(ev);
    }

    pub fn update(&mut self) {
        // FIXME: use real dt
        self.dt = std::time::Duration::from_nanos(1_000_000_000 / 60);

        // input
        self.vi.dir.update(&self.input, self.dt);
        // rendering state
        self.fov_render.update(self.dt);
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
    pub fow: FowData,
    pub entities: Vec<Player>,
}

/// Lifecycle
impl World {
    pub fn from_tiled_file(wcx: &mut WorldContext, path: &Path) -> anyhow::Result<Self> {
        let map = TiledRlMap::from_tiled_path(path)?;

        let mut entities = Vec::with_capacity(20);

        entities.push({
            let pos = Vec2i::new(14, 12);
            let mut player = Player {
                pos,
                dir: Dir8::N,
                fov: FovData::new(crate::consts::FOV_R, 10),
                img: ActorImage::from_path(asset::path("ika-chan.png"), pos, dir)?,
            };

            self::update_fov(
                &mut player.fov,
                player.pos,
                crate::consts::FOV_R,
                &map.rlmap,
            );
            wcx.fov_render.force_set_fov(&player.fov);

            player
        });

        entities.push({
            let pos = Vec2i::new(20, 15);
            let dir = Dir8::S;
            Player {
                pos,
                dir,
                fov: FovData::empty(),
                img: ActorImage::from_path(asset::path("ika-chan.png"), pos, dir)?,
            }
        });

        entities.push({
            let pos = Vec2i::new(20, 18);
            let dir = Dir8::S;
            Player {
                pos,
                dir,
                fov: FovData::empty(),
                img: ActorImage::from_path(asset::path("ika-chan.png"), pos, dir)?,
            }
        });

        let size = map.rlmap.size;
        Ok(Self {
            map,
            fow: FowData::new(size),
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

    pub fn on_end_frame(&mut self, wcx: &mut WorldContext) {
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

        false
    }
}

fn update_fov(fov: &mut impl FovWrite, pos: Vec2i, r: u32, map: &impl OpacityMap) {
    rl::fov::refresh(
        fov,
        rl::fov::RefreshParams {
            r,
            origin: pos,
            opa: map,
        },
    );
}
