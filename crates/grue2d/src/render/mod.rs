/*!
Renders the game world
*/

mod screen;
pub use screen::*;

use {
    rlbox::{render::tiled as tiled_render, utils::DoubleTrack},
    rokol::gfx as rg,
    snow2d::{
        gfx::{draw::*, Color, PassConfig, Snow2d},
        Ice,
    },
    std::time::Duration,
};

use crate::rl::world::World;

/// TODO: remove
const WALK_TIME: f32 = 8.0 / 60.0;

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

/// Renders map, actors, shadows and snow
#[derive(Debug)]
pub struct WorldRenderer {
    pub shadow_render: ShadowRenderer,
    pub snow_render: SnowRenderer,
    pa_blue: rg::PassAction,
    /// FIXME: this is inaccurate on actor insertion/deletion
    actor_visibilities: Vec<DoubleTrack<bool>>,
}

impl Default for WorldRenderer {
    fn default() -> Self {
        Self {
            shadow_render: ShadowRenderer::default(),
            snow_render: SnowRenderer::default(),
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            actor_visibilities: Default::default(),
        }
    }
}

impl WorldRenderer {
    pub fn post_update(&mut self, world: &World, _dt: Duration) {
        // ensure capacity
        // FIXME: this tracking is NOT always valid (at least use Index<T>)
        if world.entities.len() > self.actor_visibilities.len() {
            self.actor_visibilities
                .resize(world.entities.len() + 5, Default::default());
        }
    }

    /// Renders the world (maybe partially)
    pub fn render(&mut self, world: &World, ice: &mut Ice, flags: WorldRenderFlag) {
        if flags.contains(WorldRenderFlag::MAP | WorldRenderFlag::ACTORS) {
            // use world coordinates
            let mut screen = ice.rdr.screen(PassConfig {
                pa: &self.pa_blue,
                tfm: Some(world.cam.to_mat4()),
                shd: None,
            });

            if flags.contains(WorldRenderFlag::MAP) {
                Self::map(&mut screen, &world);
            }

            if flags.contains(WorldRenderFlag::ACTORS) {
                self.actors(&mut screen, &world, ice.dt);
            }
        }

        if flags.contains(WorldRenderFlag::SHADOW) {
            self.shadow(&mut ice.rdr, world);
        }

        if flags.contains(WorldRenderFlag::SNOW) {
            self.snow();
        }
    }

    fn map(screen: &mut impl DrawApi, world: &World) {
        tiled_render::render_tiled(
            screen,
            &world.map.tiled,
            &world.map.idmap,
            world.cam.bounds(),
        );

        // FIXME: can't draw rects
        // rlbox::render::tiled::render_rects_on_non_blocking_cells(
        //     screen,
        //     &world.map.tiled,
        //     &world.map.rlmap.blocks,
        //     &bounds, // FIXME: copy
        // );
    }

    // TODO: maybe use camera matrix
    fn actors(&mut self, screen: &mut impl DrawApi, world: &World, dt: Duration) {
        // FIXME: separate update and render
        // TODO: y sort + culling
        for (i, e) in world.entities.iter() {
            let x = &mut self.actor_visibilities[i.slot() as usize];

            let is_visible = world.shadow.fov.a.is_in_view(e.pos);
            if is_visible != x.a {
                x.b = x.a;
                x.a = is_visible;
                x.t = Default::default();
            }

            let max = WALK_TIME;

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
            // let pos = world.cam.w2s(e.img.render_pos_world(&world.map.tiled));
            let pos = e.img.render_pos_world(&world.map.tiled);

            screen
                .sprite(e.img.sprite())
                .dst_pos_px(pos)
                .color(Color::WHITE.with_alpha(alpha as u8));
        }
    }

    fn shadow(&mut self, rdr: &mut Snow2d, world: &World) {
        let blur = true;
        self.shadow_render.render_ofs(rdr, world, blur);
        self.shadow_render.blend_to_screen(rdr);
    }

    fn snow(&mut self) {
        self.snow_render.render();
    }
}
