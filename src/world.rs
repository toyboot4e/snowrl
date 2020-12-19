use {
    rlbox::{
        render::tiled as tiled_render,
        rl::{self, fov::FovData, fow::FowData, grid2d::*, rlmap::TiledRlMap},
    },
    rokol::{app as ra, gfx as rg},
    snow2d::{
        gfx::{batcher::draw::*, geom2d::*, Color},
        PassConfig, RenderTexture, Snow2d,
    },
    std::path::{Path, PathBuf},
};

use crate::render::FovRenderer;

/// Powers the game [`World`]
#[derive(Debug)]
pub struct WorldContext {
    /// 2D renderer
    pub rdr: Snow2d,
    /// Clears target (frame buffer) with cornflower blue color
    pa_blue: rg::PassAction,
    pub fov_render: FovRenderer,
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
        }
    }

    pub fn update(&mut self) {}
}

/// The game world
#[derive(Debug)]
pub struct World {
    pub map: TiledRlMap,
    pub fow: FowData,
    pub player: Player,
}

impl World {
    pub fn from_tiled_file(path: &Path) -> anyhow::Result<Self> {
        let map = TiledRlMap::from_tiled_path(path)?;
        let size = map.rlmap.size;

        Ok(Self {
            map,
            fow: FowData::new(size),
            player: Player {
                pos: [14, 12].into(),
                fov: FovData::new(6, 12),
            },
        })
    }

    pub fn update(&mut self, _wcx: &mut WorldContext) {
        rl::fov::refresh(
            &mut self.player.fov,
            rl::fov::RefreshParams {
                r: 6,
                origin: self.player.pos,
                opa: &self.map.rlmap,
            },
        );
    }

    pub fn render(&mut self, wcx: &mut WorldContext) {
        wcx.fov_render.render(&mut wcx.rdr, self);

        let mut screen = wcx.rdr.screen(PassConfig {
            pa: &wcx.pa_blue,
            tfm: None,
            pip: None,
        });

        crate::render::render_tiled(&mut screen, self);

        drop(screen);

        wcx.fov_render.blend_to_screen(&mut wcx.rdr);
    }
}

#[derive(Debug)]
pub struct Player {
    pub pos: Vec2i,
    pub fov: FovData,
}
