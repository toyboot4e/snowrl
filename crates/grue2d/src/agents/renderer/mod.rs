/*!
Renders the game world
*/

mod screen;
use screen::*;

use {
    rlbox::{render::tiled as tiled_render, rl::grid2d::Vec2i, utils::DoubleTrack},
    rokol::gfx as rg,
    snow2d::{
        gfx::{draw::*, Color, PassConfig, Snow2d, WindowState},
        utils::arena::Index,
        Ice,
    },
    std::time::Duration,
};

use crate::data::world::{actor::Actor, World};

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

/// Sort actors based on position
#[derive(Debug, PartialEq, Eq)]
struct ActorSortEntry {
    pub actor_index: Index<Actor>,
    pub pos: Vec2i,
}

impl ActorSortEntry {
    pub fn cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        /// NOTE: We're assuming we won't create a map with width bigger than 10,000
        fn to_cmp(v: Vec2i) -> i32 {
            v.x + v.y * 10000
        }
        to_cmp(a.pos).cmp(&to_cmp(b.pos))
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
    sort_buf: Vec<ActorSortEntry>,
}

impl WorldRenderer {
    pub fn new(screen_size: [u32; 2]) -> Self {
        Self {
            shadow_render: ShadowRenderer::new(screen_size),
            snow_render: SnowRenderer::default(),
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            actor_visibilities: Default::default(),
            sort_buf: Vec::with_capacity(32),
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
        let dt = ice.dt();
        if flags.contains(WorldRenderFlag::MAP | WorldRenderFlag::ACTORS) {
            // use world coordinates
            let mut screen = ice.snow.screen(PassConfig {
                pa: &self.pa_blue,
                tfm: Some(world.cam.to_mat4()),
                shd: None,
            });

            if flags.contains(WorldRenderFlag::MAP) {
                Self::render_map(&mut screen, &world, 0..100);
            }

            self.update_actor_images(&world, dt);
            if flags.contains(WorldRenderFlag::ACTORS) {
                self.render_actors(&mut screen, &world);
            }

            if flags.contains(WorldRenderFlag::MAP) {
                Self::render_map(&mut screen, &world, 100..);
            }
        }

        if flags.contains(WorldRenderFlag::SHADOW) {
            self.render_shadow(&mut ice.snow, world);
        }

        if flags.contains(WorldRenderFlag::SNOW) {
            self.render_snow(&ice.snow.window);
        }
    }

    fn render_map(
        screen: &mut impl DrawApi,
        world: &World,
        layer_range: impl std::ops::RangeBounds<i32>,
    ) {
        tiled_render::render_tiled(
            screen,
            &world.map.tiled,
            &world.map.idmap,
            world.cam.bounds(),
            layer_range,
        );
    }

    // TODO: fn render_map_cell_rects(
    // rlbox::render::tiled::render_rects_on_non_blocking_cells(
    //     screen,
    //     &world.map.tiled,
    //     &world.map.rlmap.blocks,
    //     &bounds,
    // );

    fn update_actor_images(&mut self, world: &World, dt: Duration) {
        self.sort_buf.clear();

        // cull and sort actors, updating interpolation value
        for (index, actor) in world.entities.iter() {
            // v: visibility (a: current, b: previous)
            let v = &mut self.actor_visibilities[index.slot() as usize];

            // TODO: cull actors based on scroll

            // update interpolation value
            {
                let is_visible = world.shadow.fov.a.is_in_view(actor.pos);

                // on visibility change
                if is_visible != v.a {
                    v.b = v.a;
                    v.a = is_visible;
                    v.t = 0.0;
                }

                // tick interpolation value
                let max = WALK_TIME;
                v.t += dt.as_secs_f32() / max;
                if v.t > 1.0 {
                    v.t = 1.0;
                }
            }

            self.sort_buf.push(ActorSortEntry {
                actor_index: index,
                pos: actor.pos,
            });
        }
    }

    /// Render actors in world coordinates. Call `update_actor_images` first
    fn render_actors(&mut self, screen: &mut impl DrawApi, world: &World) {
        self.sort_buf.sort_by(ActorSortEntry::cmp);

        for entry in &self.sort_buf {
            let alpha = {
                fn b2f(b: bool) -> f32 {
                    if b {
                        255.0
                    } else {
                        0.0
                    }
                }

                let v = &mut self.actor_visibilities[entry.actor_index.slot() as usize];
                b2f(v.a) * v.t + b2f(v.b) * (1.0 - v.t)
            } as u8;

            let actor = &world.entities[entry.actor_index];
            let pos = actor.img.pos_world_render(&world.map.tiled);

            screen
                .sprite(actor.img.sprite())
                .dst_pos_px(pos)
                .color(Color::WHITE.with_alpha(alpha));
        }
    }

    fn render_shadow(&mut self, rdr: &mut Snow2d, world: &World) {
        let blur = true;
        self.shadow_render.render_ofs(rdr, world, blur);
        self.shadow_render.blend_to_screen(rdr, &world.cam);
    }

    fn render_snow(&mut self, window: &WindowState) {
        self.snow_render.render(window);
    }
}
