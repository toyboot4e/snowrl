/*!
Renderers
*/

use {
    rlbox::{render::tiled as tiled_render, utils::DoubleTrack},
    rokol::{
        app as ra,
        gfx::{self as rg, BakedResource},
    },
    snow2d::{
        gfx::{
            batch::{mesh::DynamicMesh, vertex::VertexData},
            draw::*,
            geom2d::*,
            tex::RenderTexture,
            Color, PassConfig, Snow2d,
        },
        Ice,
    },
    std::time::{Duration, Instant},
};

use crate::{rl::world::World, Global};

/// TODO: remove
const WALK_TIME: f32 = 8.0 / 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderLayer {
    Map,
    Actors,
    Shadow,
    Snow,
    Ui,
}

#[derive(Debug)]
pub struct ShadowRenderer {
    /// Shadow textures for gaussian blur
    shadows: [RenderTexture; 2],
    /// Pipeline object for off-screen rendering with gausssian blur
    pip_gauss_ofs: rg::Pipeline,
}

impl Default for ShadowRenderer {
    fn default() -> Self {
        Self {
            shadows: [Self::create_shadow(), Self::create_shadow()],
            pip_gauss_ofs: rg::Pipeline::create(&rg::PipelineDesc {
                shader: snow2d::gfx::shaders::gauss(),
                index_type: rg::IndexType::UInt16 as u32,
                layout: VertexData::layout_desc(),
                blend: rg::BlendState {
                    // for off-screen rendering:
                    depth_format: rg::PixelFormat::Depth as u32,
                    ..Default::default()
                },
                rasterizer: rg::RasterizerState {
                    // NOTE: our renderer may output backward triangle
                    cull_mode: rg::CullMode::None as u32,
                    ..Default::default()
                },
                ..Default::default()
            }),
        }
    }
}

impl ShadowRenderer {
    /// Creates 1/4 off-screern rendering target
    fn create_shadow() -> RenderTexture {
        let inv_scale = 4.0;
        let mut screen_size = ra::size_scaled();
        screen_size[0] /= inv_scale;
        screen_size[1] /= inv_scale;
        RenderTexture::new(screen_size[0] as u32, screen_size[1] as u32)
    }

    /// Renders shadow texture (don't forget to use it later)
    pub fn render_ofs(&mut self, rdr: &mut Snow2d, world: &World) {
        // get shadow
        let mut offscreen = rdr.offscreen(
            &self.shadows[0],
            PassConfig {
                pa: &rg::PassAction::NONE,
                tfm: None,
                pip: None,
            },
        );

        // TODO: use camera
        let bounds = Rect2f::from(([0.0, 0.0], ra::size_scaled()));

        tiled_render::render_fov_fow_blend(
            &mut offscreen,
            &world.map.tiled,
            &bounds,
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
                pip: Some(self.pip_gauss_ofs),
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
            .dst_size_px(ra::size_scaled());
    }

    /// Writes shadow to the screen frame buffer
    pub fn blend_to_screen(&self, rdr: &mut Snow2d) {
        let mut screen = rdr.screen(PassConfig {
            pa: &rg::PassAction::NONE,
            tfm: None,
            pip: None,
        });

        self.blend_to_target(&mut screen);

        drop(screen);
    }

    /// Writes shadow to the screen frame buffer
    pub fn blend_to_target(&self, target: &mut impl DrawApi) {
        target
            .sprite(self.shadows[0].tex())
            .dst_size_px(ra::size_scaled());
    }
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
struct PosUvVert {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

impl PosUvVert {
    pub fn layout_desc() -> rg::LayoutDesc {
        let mut desc = rg::LayoutDesc::default();
        desc.attrs[0].format = rg::VertexFormat::Float2 as u32;
        desc.attrs[1].format = rg::VertexFormat::Float2 as u32;
        desc
    }
}

#[derive(Debug)]
pub struct SnowRenderer {
    pip_snow: rg::Pipeline,
    start_time: Instant,
    mesh: StaticMesh<PosUvVert>,
}

impl Default for SnowRenderer {
    fn default() -> Self {
        // NOTE: this is only for OpenGL
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
            pip_snow: rg::Pipeline::create(&rg::PipelineDesc {
                shader: snow2d::gfx::shaders::snow(),
                index_type: rg::IndexType::UInt16 as u32,
                layout: PosUvVert::layout_desc(),
                blend: rg::BlendState {
                    // alpha blending for on-screen rendering
                    enabled: true,
                    src_factor_rgb: rg::BlendFactor::SrcAlpha as u32,
                    dst_factor_rgb: rg::BlendFactor::OneMinusSrcAlpha as u32,
                    src_factor_alpha: rg::BlendFactor::One as u32,
                    dst_factor_alpha: rg::BlendFactor::Zero as u32,
                    ..Default::default()
                },
                rasterizer: rg::RasterizerState {
                    // NOTE: our renderer may output backward triangle
                    cull_mode: rg::CullMode::None as u32,
                    ..Default::default()
                },
                ..Default::default()
            }),
            start_time: Instant::now(),
            mesh: StaticMesh::new_16(&verts, &[0, 1, 2]),
        }
    }
}

impl SnowRenderer {
    pub fn render(&mut self) {
        rg::begin_default_pass(&rg::PassAction::NONE, ra::width(), ra::height());
        rg::apply_pipeline(self.pip_snow);

        // set uniforms
        unsafe {
            // FS
            let res = glam::Vec2::from(ra::size_scaled());
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Fs, 0, &res);

            let time = (Instant::now() - self.start_time).as_secs_f32();
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Fs, 1, &time);

            let mouse = glam::Vec2::new(ra::width() as f32, ra::height() as f32);
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Fs, 2, &mouse);
        }

        // just draw a fullscreen triangle
        self.mesh.draw();

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
        // resize to ensure capacity
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
                pip: None,
            });

            if flags.contains(WorldRenderFlag::MAP) {
                Self::map(&mut screen, &world);
            }

            if flags.contains(WorldRenderFlag::ACTORS) {
                self.actors(&mut screen, &world, ice.dt);
            }
        }

        if flags.contains(WorldRenderFlag::SHADOW) {
            self.shadow_render.render_ofs(&mut ice.rdr, &world);
            self.shadow(&mut ice.rdr);
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

            e.img
                .render(screen, &world.map.tiled)
                .color(Color::WHITE.with_alpha(alpha as u8));
        }
    }

    fn shadow(&mut self, rdr: &mut Snow2d) {
        self.shadow_render.blend_to_screen(rdr);
    }

    fn snow(&mut self) {
        self.snow_render.render();
    }
}
