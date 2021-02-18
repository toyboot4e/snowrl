/*!
Renderers
*/

use {
    rlbox::{render::tiled as tiled_render, utils::DoubleTrack},
    rokol::{app as ra, gfx as rg},
    snow2d::{
        gfx::{
            draw::*, mesh::StaticMesh, shaders, shaders::PosUvVert, tex::RenderTexture, Color,
            PassConfig, Shader, Snow2d,
        },
        Ice,
    },
    std::time::{Duration, Instant},
};

use crate::rl::world::World;

/// TODO: remove
const WALK_TIME: f32 = 8.0 / 60.0;

// /// TODO: use it?
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum RenderLayer {
//     Map,
//     Actors,
//     Shadow,
//     Snow,
//     Ui,
// }

/// Renders FoV/FoW shadows
#[derive(Debug)]
pub struct ShadowRenderer {
    /// Shadow textures for gaussian blur
    shadows: [RenderTexture; 2],
    /// Shader program for off-screen rendering with gausssian blur
    gauss_shd: Shader,
}

impl Default for ShadowRenderer {
    fn default() -> Self {
        Self {
            shadows: [Self::create_shadow(), Self::create_shadow()],
            gauss_shd: shaders::gauss(),
        }
    }
}

impl ShadowRenderer {
    /// Creates 1/4 off-screern rendering target
    fn create_shadow() -> RenderTexture {
        let inv_scale = 4.0;
        let mut screen_size = ra::size_f_scaled();
        screen_size[0] /= inv_scale;
        screen_size[1] /= inv_scale;
        RenderTexture::builder([screen_size[0] as u32, screen_size[1] as u32])
            // Linear filter is smoother
            .filter(rg::Filter::Nearest)
            .build()
    }

    /// Renders shadow texture (don't forget to use it later)
    pub fn render_ofs(&mut self, rdr: &mut Snow2d, world: &World) {
        let mut offscreen = rdr.offscreen(
            &mut self.shadows[0],
            PassConfig {
                pa: &rg::PassAction::NONE,
                tfm: None,
                shd: None,
            },
        );

        tiled_render::render_fov_fow_blend(
            &mut offscreen,
            &world.map.tiled,
            &world.cam.bounds(),
            &world.shadow.fov.a,
            &world.shadow.fov.b,
            world.shadow.dt.get(),
            &world.shadow.fow.a,
            &world.shadow.fow.b,
        );

        drop(offscreen);

        // apply gaussian blur
        self.pingpong(rdr);
    }

    /// Apply gaussian blur
    fn pingpong(&mut self, rdr: &mut Snow2d) {
        // 5 times
        for _ in 0..5 {
            // pingpong blur
            for ix in 0..2 {
                // (i, j) = (source, target)
                let i = ix % 2;
                let j = (ix + 1) % 2;
                self.blur(rdr, ix == 0, i, j);
            }
        }
    }

    #[inline]
    fn blur(&mut self, rdr: &mut Snow2d, is_h: bool, from: usize, to: usize) {
        let mut draw = rdr.offscreen(
            &mut self.shadows[to],
            PassConfig {
                pa: &rg::PassAction::NONE,
                tfm: None,
                shd: Some(&self.gauss_shd),
            },
        );

        // horizontally or vertically
        unsafe {
            let ub_index = 1;
            let uniform: f32 = if is_h { 1.0 } else { 0.0 };
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, ub_index, &uniform);
        }

        // write from one to the other
        draw.sprite(self.shadows[from].tex())
            // NOTE: we're using a orthogarphic projection matrix for the screen, so
            // use the screen size as the destination size
            .dst_size_px(ra::size_f_scaled());
    }

    /// Writes shadow to the screen frame buffer
    pub fn blend_to_screen(&self, rdr: &mut Snow2d) {
        let mut screen = rdr.screen(PassConfig {
            pa: &rg::PassAction::NONE,
            tfm: None,
            shd: None,
        });

        self.blend_to_target(&mut screen);

        drop(screen);
    }

    /// Writes shadow to the screen frame buffer
    pub fn blend_to_target(&self, target: &mut impl DrawApi) {
        target
            .sprite(self.shadows[0].tex())
            .dst_size_px(ra::size_f_scaled());
    }
}

/// Renders snow on fullscreen
#[derive(Debug)]
pub struct SnowRenderer {
    shd: Shader,
    start_time: Instant,
    mesh: StaticMesh<PosUvVert>,
}

impl Default for SnowRenderer {
    fn default() -> Self {
        // NOTE: this works only for OpenGL
        let verts = vec![
            PosUvVert {
                pos: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            PosUvVert {
                pos: [3.0, -1.0],
                uv: [2.0, 0.0],
            },
            PosUvVert {
                pos: [-1.0, 3.0],
                uv: [0.0, 2.0],
            },
        ];

        Self {
            shd: shaders::snow(),
            start_time: Instant::now(),
            mesh: StaticMesh::new_16(&verts, &[0, 1, 2]),
        }
    }
}

impl SnowRenderer {
    pub fn render(&mut self) {
        rg::begin_default_pass(&rg::PassAction::NONE, ra::width(), ra::height());
        self.shd.apply_pip();

        fn as_bytes<T>(x: &T) -> &[u8] {
            unsafe {
                std::slice::from_raw_parts(x as *const _ as *const _, std::mem::size_of::<T>())
            }
        }

        let size = glam::Vec2::from(ra::size_f_scaled());
        self.shd.set_fs_uniform(0, as_bytes(&size));

        let time = (Instant::now() - self.start_time).as_secs_f32();
        self.shd.set_fs_uniform(1, as_bytes(&time));

        let mouse = glam::Vec2::new(ra::width() as f32, ra::height() as f32);
        self.shd.set_fs_uniform(2, as_bytes(&mouse));

        // just draw a fullscreen triangle
        self.mesh.draw_all();

        rg::end_pass();
    }
}

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
            let mut screen = ice.rdr.screen(PassConfig {
                pa: &self.pa_blue,
                tfm: None,
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
        rlbox::render::tiled::render_tiled(
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
            let pos = world.cam.w2s(e.img.pos_world(&world.map.tiled));

            screen
                .sprite(e.img.sprite())
                // TODO: align y
                .dst_pos_px(pos)
                .color(Color::WHITE.with_alpha(alpha as u8));
        }
    }

    fn shadow(&mut self, rdr: &mut Snow2d, world: &World) {
        self.shadow_render.render_ofs(rdr, world);
        self.shadow_render.blend_to_screen(rdr);
    }

    fn snow(&mut self) {
        self.snow_render.render();
    }
}
