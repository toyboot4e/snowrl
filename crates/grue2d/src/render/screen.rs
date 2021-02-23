/*!
Full-screen enderers
*/

use {
    rokol::{app as ra, gfx as rg},
    snow2d::gfx::{
        draw::*, mesh::StaticMesh, shaders, shaders::PosUvVert, tex::RenderTexture, PassConfig,
        Shader, Snow2d,
    },
    std::time::Instant,
};

use crate::rl::world::World;
use rlbox::render::tiled as tiled_render;

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
    /// Creates off-screern rendering target
    fn create_shadow() -> RenderTexture {
        let mut screen_size = ra::size_f_scaled();

        // the smaller the fuzzyier
        let scale = 1.0 / 3.0;
        screen_size[0] *= scale;
        screen_size[1] *= scale;

        RenderTexture::builder([screen_size[0] as u32, screen_size[1] as u32])
            // linear: smooth, nearest: feels like pixelized
            // TODO: let user choose it dynamically
            .filter(rg::Filter::Nearest)
            .build()
    }

    /// Renders shadow texture (don't forget to use it later)
    pub fn render_ofs(&mut self, rdr: &mut Snow2d, world: &World, blur: bool) {
        let mut offscreen = rdr.offscreen(
            &mut self.shadows[0],
            PassConfig {
                pa: &rg::PassAction::LOAD,
                tfm: Some(world.cam.to_mat4()),
                shd: None,
            },
        );

        // get shadow texture
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

        if blur {
            // apply gaussian blur
            self.pingpong(rdr);
        }
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
                pa: &rg::PassAction::LOAD,
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
            pa: &rg::PassAction::LOAD,
            tfm: None,
            shd: None,
        });

        self.blend_to_target(&mut screen);
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
        rg::begin_default_pass(&rg::PassAction::LOAD, ra::width(), ra::height());
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
