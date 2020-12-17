use {
    rlbox::{
        render::tiled as tiled_render,
        rl::{self, fov::FovData, fow::FowData, grid2d::*, rlmap::TiledRlMap},
    },
    rokol::gfx as rg,
    snow2d::{
        gfx::{batcher::draw::*, geom2d::*, tex::Texture2dDrop},
        Snow2d,
    },
    std::path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct WorldContext {
    /// 2D renderer
    rdr: Snow2d,
}

impl WorldContext {
    pub fn new() -> Self {
        let mut rdr = Snow2d::new();
        unsafe {
            rdr.init();
        }

        Self { rdr }
    }

    pub fn update(&mut self) {}
}

#[derive(Debug)]
pub struct World {
    map: TiledRlMap,
    fow: FowData,
    player: Player,
}

impl World {
    pub fn from_tiled_file(path: &Path) -> anyhow::Result<Self> {
        let map = TiledRlMap::from_tiled_path(path)?;

        let size = map.rlmap.size;
        Ok(Self {
            map,
            fow: FowData::new(size),
            player: Player {
                pos: [10, 10].into(),
                fov: FovData::new(6, 12),
            },
        })
    }

    pub fn update(&mut self, wcx: &mut WorldContext) {
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
        let mut batch = wcx.rdr.begin_default_pass();

        let bounds = Rect2f::from(([0.0, 0.0], [1280.0, 720.0]));
        tiled_render::render_tiled(&mut batch, &self.map.tiled, &self.map.idmap, bounds.clone());

        tiled_render::render_fov_shadows(&mut batch, &self.map.tiled, &self.player.fov, &bounds);
    }
}

#[derive(Debug)]
pub struct Player {
    pos: Vec2i,
    fov: FovData,
}
