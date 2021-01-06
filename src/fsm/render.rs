use {
    rokol::{app as ra, gfx as rg},
    snow2d::{
        gfx::{batcher::draw::*, geom2d::*, Color},
        PassConfig, Snow2d,
    },
};

use crate::{
    fsm::Global,
    world::{render::*, World, WorldContext},
};

bitflags::bitflags! {
    /// Fixed set of renderers
    pub struct WorldRenderFlag: u32 {
        const SHADOW = 1 << 0;
        const SNOW = 1 << 1;
        const RL_ANIMS = 1 << 2;
        const ACTORS = 1 << 3;
        const MAP = 1 << 4;
        //
        const ALL = Self::SHADOW.bits | Self::SNOW.bits | Self::RL_ANIMS.bits | Self::ACTORS.bits | Self::MAP.bits;
    }
}

#[derive(Debug)]
pub struct WorldRenderer {
    pub fov_render: FovRenderer,
    pub snow_render: SnowRenderer,
    pa_blue: rg::PassAction,
}

impl Default for WorldRenderer {
    fn default() -> Self {
        Self {
            fov_render: FovRenderer::default(),
            snow_render: SnowRenderer::default(),
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
        }
    }
}

impl WorldRenderer {
    pub fn update(&mut self, gl: &Global) {
        //
    }

    /// Renders the world (maybe partially)
    pub fn render(&mut self, world: &World, wcx: &mut WorldContext, flags: WorldRenderFlag) {
        if flags.contains(WorldRenderFlag::MAP | WorldRenderFlag::ACTORS) {
            let mut screen = wcx.rdr.screen(PassConfig {
                pa: &self.pa_blue,
                tfm: None,
                pip: None,
            });

            if flags.contains(WorldRenderFlag::MAP) {
                Self::map(&mut screen, &world);
            }

            if flags.contains(WorldRenderFlag::ACTORS) {
                Self::actors(&mut screen, &world);
            }
        }

        if flags.contains(WorldRenderFlag::SHADOW) {
            self.fov_render.render_ofs(&mut wcx.rdr, &world);
            self.shadow(&mut wcx.rdr);
        }

        if flags.contains(WorldRenderFlag::SNOW) {
            self.snow();
        }
    }

    fn map(screen: &mut impl DrawApi, world: &World) {
        let bounds = Rect2f::from(([0.0, 0.0], ra::size_scaled()));

        rlbox::render::tiled::render_tiled(
            screen,
            &world.map.tiled,
            &world.map.idmap,
            bounds.clone(),
        );
    }

    fn actors(screen: &mut impl DrawApi, world: &World) {
        // TODO: y sort + culling
        for e in &world.entities {
            e.img.render(screen, &world.map.tiled);
        }
    }

    fn shadow(&mut self, rdr: &mut Snow2d) {
        self.fov_render.blend_to_screen(rdr);
    }

    fn snow(&mut self) {
        self.snow_render.render();
    }
}
