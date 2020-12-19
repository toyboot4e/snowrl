use {
    rlbox::rl::{
        self,
        fov::{FovData, FovWrite, OpacityMap},
        fow::FowData,
        grid2d::*,
        rlmap::TiledRlMap,
    },
    rokol::gfx as rg,
    snow2d::{gfx::Color, PassConfig, Snow2d},
    std::path::Path,
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
    pub fn from_tiled_file(wcx: &mut WorldContext, path: &Path) -> anyhow::Result<Self> {
        let map = TiledRlMap::from_tiled_path(path)?;
        let size = map.rlmap.size;

        let mut player = Player {
            pos: [14, 12].into(),
            fov: FovData::new(6, 12),
        };

        Self::update_fov(&mut player.fov, player.pos, &map.rlmap);

        wcx.fov_render.set_prev_fov(&player.fov);

        Ok(Self {
            map,
            fow: FowData::new(size),
            player,
        })
    }

    fn update_fov(fov: &mut impl FovWrite, pos: Vec2i, map: &impl OpacityMap) {
        rl::fov::refresh(
            fov,
            rl::fov::RefreshParams {
                r: 6,
                origin: pos,
                opa: map,
            },
        );
    }

    pub fn update(&mut self, _wcx: &mut WorldContext) {
        //
    }

    pub fn render(&mut self, wcx: &mut WorldContext) {
        let mut screen = wcx.rdr.screen(PassConfig {
            pa: &wcx.pa_blue,
            tfm: None,
            pip: None,
        });
        crate::render::render_tiled(&mut screen, self);
        drop(screen);

        wcx.fov_render.render_ofs(&mut wcx.rdr, self);
        wcx.fov_render.blend_to_screen(&mut wcx.rdr);
    }
}

#[derive(Debug)]
pub struct Player {
    pub pos: Vec2i,
    pub fov: FovData,
}
