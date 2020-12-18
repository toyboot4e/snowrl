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

/// Powers the game [`World`]
#[derive(Debug)]
pub struct WorldContext {
    /// 2D renderer
    rdr: Snow2d,
    /// Clears target (frame buffer) with cornflower blue color
    pa_blue: rg::PassAction,
    /// Clears target (shadow) with
    pa_trans: rg::PassAction,
    shadow: RenderTexture,
}

impl WorldContext {
    pub fn new() -> Self {
        let mut rdr = Snow2d::new();
        unsafe {
            rdr.init();
        }

        let shadow = {
            let inv_scale = 4.0;
            let mut screen_size = ra::size_scaled();
            screen_size[0] /= inv_scale;
            screen_size[1] /= inv_scale;
            RenderTexture::new(screen_size[0] as u32, screen_size[1] as u32)
        };

        Self {
            rdr,
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            // pa_trans: rg::PassAction::clear(Color::WHITE_TRANSPARENT.to_normalized_array()),
            pa_trans: rg::PassAction::clear(Color::BLACK.to_normalized_array()),
            shadow,
        }
    }

    pub fn update(&mut self) {}
}

/// The game world
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
        // TODO: 1/4 size for blur
        let mut offscreen = wcx.rdr.offscreen(
            &wcx.shadow,
            PassConfig {
                pa: &wcx.pa_trans,
                tfm: None,
                state: None,
            },
        );

        let bounds = Rect2f::from(([0.0, 0.0], ra::size_scaled()));
        tiled_render::render_fov_shadows(
            &mut offscreen,
            &self.map.tiled,
            &self.player.fov,
            &bounds,
        );

        drop(offscreen);

        let mut screen = wcx.rdr.screen(PassConfig {
            pa: &wcx.pa_blue,
            tfm: None,
            state: None,
        });

        tiled_render::render_tiled(
            &mut screen,
            &self.map.tiled,
            &self.map.idmap,
            bounds.clone(),
        );

        screen
            .sprite(wcx.shadow.tex())
            .dst_size_px(ra::size_scaled());

        drop(screen);
    }
}

#[derive(Debug)]
pub struct Player {
    pos: Vec2i,
    fov: FovData,
}
