/*!
Renders the game world
*/

mod screen;
use screen::*;

use {
    rlbox::{render::tiled as tiled_render, rl::grid2d::Vec2i, utils::DoubleTrack},
    snow2d::{
        gfx::{draw::*, Color, GameClock, Snow2d, WindowState},
        ui::Ui,
        utils::arena::Index,
    },
    std::time::Duration,
};

use crate::game::{
    cfg::{ShadowConfig, SnowConfig},
    data::world::{actor::Actor, World},
};

/// TODO: remove
const WALK_TIME: f32 = 8.0 / 60.0;

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

/// Renders map, shadows and snow. Also sets up actor nodes
#[derive(Debug)]
pub struct WorldRenderer {
    pub shadow_render: ShadowRenderer,
    pub snow_render: SnowRenderer,
    /// FIXME: this is inaccurate on actor insertion/deletion
    actor_visibilities: Vec<DoubleTrack<bool>>,
    sort_buf: Vec<ActorSortEntry>,
}

impl WorldRenderer {
    pub fn new(screen_size: [u32; 2], clock: &GameClock) -> Self {
        Self {
            shadow_render: ShadowRenderer::new(screen_size),
            snow_render: SnowRenderer::new(clock),
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

    pub fn render_map(
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

    fn actor_alpha_f32(&self, slot: usize) -> f32 {
        fn b2f(b: bool) -> f32 {
            if b {
                255.0
            } else {
                0.0
            }
        }

        let v = &self.actor_visibilities[slot];
        b2f(v.a) * v.t + b2f(v.b) * (1.0 - v.t)
    }

    pub fn setup_actor_nodes(&mut self, world: &World, ui: &mut Ui, dt: Duration) {
        self.update_actor_images(&world, dt);
        self.sort_buf.sort_by(ActorSortEntry::cmp);

        let n_entries = self.sort_buf.len() as f32;
        for (entry_ix, entry) in self.sort_buf.iter().enumerate() {
            let actor = &world.entities[entry.actor_index];

            let alpha = self.actor_alpha_f32(entry.actor_index.slot() as usize) as u8;

            let base_node = &mut ui.nodes[&actor.nodes.base];
            base_node.z_order = entry_ix as f32 / n_entries;
            base_node.params.pos = actor.view.base_pos_world(&world.map.tiled);

            let img_node = &mut ui.nodes[&actor.nodes.img];
            img_node.z_order = entry_ix as f32 / n_entries;
            // NOTE: here we're animationg the actor image
            img_node.surface = actor.view.sprite().into();
            img_node.params.color = Color::WHITE.with_alpha(alpha);
        }
    }

    /// FIXME: Don't re-create shadow when not needed
    pub fn render_shadow(&mut self, rdr: &mut Snow2d, world: &World, cfg: &ShadowConfig) {
        match cfg {
            ShadowConfig::Blur => {
                let blur = true;
                self.shadow_render.render_ofs(rdr, world, blur);
                self.shadow_render.blend_to_screen(rdr, &world.cam);
            }
            ShadowConfig::Raw => {
                let blur = false;
                self.shadow_render.render_ofs(rdr, world, blur);
                self.shadow_render.blend_to_screen(rdr, &world.cam);
            }
            ShadowConfig::None => {
                //
            }
        }
    }

    pub fn render_snow(&mut self, window: &WindowState, clock: &GameClock, cfg: &SnowConfig) {
        match cfg {
            SnowConfig::Blizzard => {
                self.snow_render.render(window, clock);
            }
            SnowConfig::None => {}
        }
    }
}
