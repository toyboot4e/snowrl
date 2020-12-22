//! The game world

pub mod actor;
mod vi;

use {
    rokol::gfx as rg,
    std::{path::Path, time::Duration},
    xdl::Key,
};

use snow2d::{asset, gfx::Color, PassConfig, Snow2d};

use rlbox::rl::{
    self,
    fov::{FovData, FovWrite, OpacityMap},
    fow::FowData,
    grid2d::*,
    rlmap::{RlMap, TiledRlMap},
};

use crate::render::FovRenderer;

use self::{actor::*, vi::VInput};

/// Powers the game [`World`]
#[derive(Debug)]
pub struct WorldContext {
    /// 2D renderer
    pub rdr: Snow2d,
    /// Clears target (frame buffer) with cornflower blue color
    pa_blue: rg::PassAction,
    pub fov_render: FovRenderer,
    pub input: xdl::Input,
    pub vi: VInput,
    pub dt: Duration,
}

impl WorldContext {
    pub fn new() -> Self {
        let mut rdr = Snow2d::new();
        unsafe {
            rdr.init();
        }

        Self {
            rdr,
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

        self.vi.dir.update(&self.input, self.dt);

        self.fov_render.update();
    }

    pub fn render(&mut self) {}

    pub fn on_end_frame(&mut self) {
        self.input.on_end_frame();
    }
}

/// The game world
#[derive(Debug)]
pub struct World {
    pub map: TiledRlMap,
    pub fow: FowData,
    pub player: Player,
}

/// Lifecycle
impl World {
    pub fn from_tiled_file(wcx: &mut WorldContext, path: &Path) -> anyhow::Result<Self> {
        let map = TiledRlMap::from_tiled_path(path)?;
        let size = map.rlmap.size;

        let pos = Vec2i::new(14, 12);
        let mut player = Player {
            pos,
            dir: Dir8::N,
            fov: FovData::new(crate::consts::FOV_R, 10),
            img: ActorImage::from_path(asset::path("ika-chan.png"), pos, Dir8::N)?,
        };

        Self::update_fov(
            &mut player.fov,
            player.pos,
            crate::consts::FOV_R,
            &map.rlmap,
        );
        wcx.fov_render.force_set_fov(&player.fov);

        Ok(Self {
            map,
            fow: FowData::new(size),
            player,
        })
    }

    pub fn event(&mut self, wcx: &mut WorldContext, ev: &rokol::app::Event) {}

    pub fn update(&mut self, wcx: &mut WorldContext) {
        if let Some(dir) = wcx.vi.dir.to_dir8() {
            Self::walk(&mut self.player, &self.map.rlmap, wcx, dir);
        }

        self.player
            .img
            .update(wcx.dt, self.player.pos, self.player.dir);
    }

    pub fn render(&mut self, wcx: &mut WorldContext) {
        let mut screen = wcx.rdr.screen(PassConfig {
            pa: &wcx.pa_blue,
            tfm: None,
            pip: None,
        });

        crate::render::render_tiled(&mut screen, self);
        self.player.img.render(&mut screen, &self.map.tiled);

        drop(screen);

        wcx.fov_render.render_ofs(&mut wcx.rdr, self);
        wcx.fov_render.blend_to_screen(&mut wcx.rdr);
    }

    pub fn on_end_frame(&mut self, wcx: &mut WorldContext) {}
}

/// Internals
impl World {
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

    fn walk(player: &mut Player, rlmap: &RlMap, wcx: &mut WorldContext, dir: Dir8) {
        let pos = player.pos + Vec2i::from(dir.signs_i32());
        if !rlmap.is_blocked(pos) {
            if wcx
                .input
                .kbd
                .is_any_key_down(&[Key::LeftShift, Key::RightShift])
            {
                // rotate only
                player.dir = dir;
            } else {
                // TODO: use command pattern. ActorIndex.. ECS?
                wcx.fov_render.before_update_fov(&player.fov);

                player.pos = pos;
                player.dir = dir;

                Self::update_fov(&mut player.fov, player.pos, crate::consts::FOV_R, rlmap);
            }
        }
    }
}
