//! Screen renderers

use {
    rlbox::utils::DoubleTrack,
    rokol::{app as ra, gfx as rg},
    snow2d::gfx::{draw::*, geom2d::*, Color, PassConfig, Snow2d},
    std::time::Duration,
};

use crate::world::{render::*, World, WorldContext};

bitflags::bitflags! {
    /// Fixed set of renderers
    pub struct WorldRenderFlag: u32 {
        const SHADOW = 1 << 0;
        const SNOW = 1 << 1;
        const ACTORS = 1 << 3;
        const MAP = 1 << 4;
        //
        const ALL = Self::SHADOW.bits | Self::SNOW.bits |  Self::ACTORS.bits | Self::MAP.bits;
    }
}

#[derive(Debug)]
pub struct WorldRenderer {
    pub fov_render: FovRenderer,
    pub snow_render: SnowRenderer,
    pa_blue: rg::PassAction,
    actor_visibilities: Vec<DoubleTrack<bool>>,
}

impl Default for WorldRenderer {
    fn default() -> Self {
        Self {
            fov_render: FovRenderer::default(),
            snow_render: SnowRenderer::default(),
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            actor_visibilities: Default::default(),
        }
    }
}

impl WorldRenderer {
    pub fn post_update(&mut self, world: &World, _dt: Duration) {
        // resize to ensure capacity
        if world.entities.len() > self.actor_visibilities.len() {
            self.actor_visibilities
                .resize(world.entities.len() + 5, Default::default());
        }
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
                self.actors(&mut screen, &world, wcx.dt);
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

        // FIXME: can't draw rects
        // rlbox::render::tiled::render_rects_on_non_blocking_cells(
        //     screen,
        //     &world.map.tiled,
        //     &world.map.rlmap.blocks,
        //     &bounds, // FIXME: copy
        // );
    }

    fn actors(&mut self, screen: &mut impl DrawApi, world: &World, dt: Duration) {
        // FIXME: separate update and render
        // TODO: y sort + culling
        for (i, e) in world.entities.iter().enumerate() {
            let x = &mut self.actor_visibilities[i];

            let is_visible = world.shadow.fov.a.is_in_view(e.pos);
            if is_visible != x.a {
                x.b = x.a;
                x.a = is_visible;
                x.t = Default::default();
            }

            let max = crate::consts::WALK_TIME;

            x.t += dt.as_secs_f32() / max;
            if x.t > 1.0 {
                x.t = 1.0;
            }

            fn b2f(b: bool) -> f32 {
                if b {
                    255.0
                } else {
                    0.0
                }
            }

            let alpha = b2f(x.a) * x.t + b2f(x.b) * (1.0 - x.t);

            e.img
                .render(screen, &world.map.tiled)
                .color(Color::WHITE.with_alpha(alpha as u8));
        }
    }

    fn shadow(&mut self, rdr: &mut Snow2d) {
        self.fov_render.blend_to_screen(rdr);
    }

    fn snow(&mut self) {
        self.snow_render.render();
    }
}
