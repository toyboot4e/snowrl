use {
    rlbox::{
        render::tiled as tiled_render,
        rl::{self, fov::FovData, fow::FowData, grid2d::*, rlmap::TiledRlMap},
    },
    rokol::{app as ra, gfx as rg},
    snow2d::{
        gfx::{batcher::draw::*, geom2d::*, tex::Texture2dDrop, Color},
        OffscreenPass, PassConfig, Snow2d,
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
    pa: rg::PassAction,
    map: TiledRlMap,
    ofs: OffscreenPass,
    fow: FowData,
    player: Player,
}

impl World {
    pub fn from_tiled_file(path: &Path) -> anyhow::Result<Self> {
        let map = TiledRlMap::from_tiled_path(path)?;
        let size = map.rlmap.size;

        Ok(Self {
            pa: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            map,
            fow: FowData::new(size),
            ofs: OffscreenPass::new(ra::width(), ra::height()),
            player: Player {
                pos: [14, 12].into(),
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
        let mut batch = wcx.rdr.begin_pass(PassConfig {
            pa: &self.pa,
            ofs: Some(&self.ofs),
        });

        let bounds = Rect2f::from(([0.0, 0.0], [1280.0, 720.0]));
        tiled_render::render_tiled(&mut batch, &self.map.tiled, &self.map.idmap, bounds.clone());

        tiled_render::render_fov_shadows(&mut batch, &self.map.tiled, &self.player.fov, &bounds);

        drop(batch);

        let mut batch = wcx.rdr.begin_pass(PassConfig {
            pa: &self.pa,
            ofs: None,
        });

        batch.sprite(self.ofs.tex());

        // let bounds = Rect2f::from(([0.0, 0.0], [1280.0, 720.0]));
        // tiled_render::render_tiled(&mut batch, &self.map.tiled, &self.map.idmap, bounds.clone());

        // tiled_render::render_fov_shadows(&mut batch, &self.map.tiled, &self.player.fov, &bounds);

        drop(batch);
    }
}

#[derive(Debug)]
pub struct Player {
    pos: Vec2i,
    fov: FovData,
}
