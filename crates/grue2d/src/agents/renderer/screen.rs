/*!
Full-screen renderers

# Pixel-perfect shadow

If our shadow is not pixel-perfect, they would be flickering or shaking. They happen because each
pixel of our shadow texture is mapped to 4x4 pixels from different cells.

So we use (screen_size / 4) shadow texture and map them to (screen_size + 1).
*/

use {
    rokol::gfx as rg,
    snow2d::{
        gfx::{
            draw::*, geom2d::Vec2f, mesh::StaticMesh, shaders, shaders::PosUvVert,
            tex::RenderTexture, Shader, Snow2d, WindowState,
        },
        utils::bytemuck,
    },
    std::time::Instant,
};

use crate::data::world::World;
use rlbox::{render::tiled as tiled_render, rl::grid2d::Vec2i, view::camera::Camera2d};

/// The smaller, the more blur
const SHADOW_SCALE: f32 = 1.0 / 4.0;

/// We'll use (screen_size + SCREEN_EDGE) for making pixel-perfect shadow
const SCREEN_EDGE: f32 = 4.0;

fn screen_to_shadow_u32(screen_size: [u32; 2]) -> [u32; 2] {
    [
        ((screen_size[0] as f32 + SCREEN_EDGE) * SHADOW_SCALE) as u32,
        ((screen_size[1] as f32 + SCREEN_EDGE) * SHADOW_SCALE) as u32,
    ]
}

const SCREEN_TRIANGLE: [PosUvVert; 3] = [
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
///
/// # How it works
///
/// 1. Calculate shadow by cell (based on some articles on Rogue Basin)
/// 2. Draw shadow to a shadow texture based on cells
/// 3. Apply Gaussian blur (shader code taken from Learn OpenGL [Bloom chapter])
/// 4. Draw the shadow to the screen
///
/// [Bloom Chapter]: https://learnopengl.com/Advanced-Lighting/Bloom
///
/// # Rendering pixel-perfect shadow when scrolling
///
/// We're using shadow textures with the size of screen size / 4. When we draw shadow to the screen,
/// it is up scaled . But then 1x1 pixel in shadow may be mapped to **4x4 pixels on screen with
/// different level of darkness/brightness**, in other words, different cells. Therefore, we need
/// some workaround to make sure every 4x4 pixels is always in one cell (e.g. when the camera
/// position is not multiples of 4).
#[derive(Debug)]
pub struct ShadowRenderer {
    /// Shadow textures for gaussian blur
    shadows: [RenderTexture; 2],
    // TODO: enable resizing.
    screen_size: [u32; 2],
    /// Shader program for off-screen rendering with gausssian blur
    gauss_shd: Shader,
    // mesh: StaticMesh<PosUvVert>,
}

impl ShadowRenderer {
    pub fn new(screen_size: [u32; 2]) -> Self {
        Self {
            shadows: [
                Self::create_shadow(screen_size),
                Self::create_shadow(screen_size),
            ],
            screen_size,
            gauss_shd: shaders::gauss(),
        }
    }
}

impl ShadowRenderer {
    /// Creates off-screern rendering target
    fn create_shadow(screen_size: [u32; 2]) -> RenderTexture {
        let shadow_size = self::screen_to_shadow_u32(screen_size);
        RenderTexture::builder([shadow_size[0] as u32, shadow_size[1] as u32])
            // linear: smooth, nearest: feels like pixelized
            // TODO: let user choose it dynamically
            .filter(rg::Filter::Nearest)
            .build()
    }

    /// Render shadow texture (don't forget to use it later)
    pub fn render_ofs(&mut self, rdr: &mut Snow2d, world: &World, blur: bool) {
        let screen_size = rdr.window.size_u32();
        if screen_size != self.screen_size {
            log::error!("The shadow size isn't synced with the screen size");
        }

        let mut offscreen = rdr
            .offscreen(&mut self.shadows[0])
            .pa(Some(&rg::PassAction::LOAD))
            .transform(Some(world.cam.to_mat4()))
            .build();

        // Use (screen_size + SCREEN_EDGE) as target size
        // (important trick for pixel-perfect shadow)
        let tfm = glam::Mat4::from_translation({
            let offset_f = world.cam.params.pos.floor();
            let offset = Vec2i::new(offset_f.x as i32, offset_f.y as i32);
            let rem = offset % 4;
            glam::Vec3::new((-offset.x + rem.x) as f32, (-offset.y + rem.y) as f32, 0.0)
        });

        // set transform matrix
        {
            const M_INV_Y: glam::Mat4 = glam::const_mat4!(
                [1.0, 0.0, 0.0, 0.0],
                [0.0, -1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            );

            let proj = M_INV_Y
                * glam::Mat4::orthographic_rh_gl(
                    // left, right
                    0.0,
                    screen_size[0] as f32,
                    // bottom, top
                    screen_size[1] as f32,
                    0.0,
                    // near, far
                    0.0,
                    1.0,
                )
                * tfm;

            rg::apply_uniforms(rg::ShaderStage::Vs, 0, bytemuck::bytes_of(&proj));
        }

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
        let mut draw = rdr
            .offscreen(&mut self.shadows[to])
            .pa(Some(&rg::PassAction::LOAD))
            .shader(Some(&self.gauss_shd))
            .build();

        // apply blur horizontally or vertically
        {
            let ub_index = 1;
            let uniform: f32 = if is_h { 1.0 } else { 0.0 };
            rg::apply_uniforms(rg::ShaderStage::Vs, ub_index, bytemuck::bytes_of(&uniform));
        }

        // write from one to the other
        draw.sprite(self.shadows[from].tex())
            // NOTE: In normalized coordinates, this is [1.0, 1.0]
            .dst_size_px([self.screen_size[0] as f32, self.screen_size[1] as f32]);
    }

    /// Writes shadow to the screen frame buffer
    pub fn blend_to_screen(&self, rdr: &mut Snow2d, cam: &Camera2d) {
        let mut screen = rdr.screen().pa(Some(&rg::PassAction::LOAD)).build();

        // NOTE: This is an important trick to create pixel-perfect shadow
        let offset_f = cam.params.pos.floor();
        let offset = Vec2i::new(offset_f.x as i32, offset_f.y as i32);
        let rem = offset % 4;
        let size = Vec2f::from([
            self.screen_size[0] as f32 + SCREEN_EDGE,
            self.screen_size[1] as f32 + SCREEN_EDGE,
        ]);

        screen
            .sprite(self.shadows[0].tex())
            .dst_pos_px([-rem.x as f32, -rem.y as f32])
            .dst_size_px(size);
    }
}

/// Renders snow on fullscreen
///
/// Uses the [Just snow] shader by baldand. Be warned that is has some restrictive license.
///
/// [Just snow]: https://www.shadertoy.com/view/ldsGDn
#[derive(Debug)]
pub struct SnowRenderer {
    shd: Shader,
    start_time: Instant,
    mesh: StaticMesh<PosUvVert>,
}

impl Default for SnowRenderer {
    fn default() -> Self {
        // NOTE: this works only for OpenGL
        Self {
            shd: shaders::snow(),
            start_time: Instant::now(),
            mesh: StaticMesh::new_16(&SCREEN_TRIANGLE, &[0, 1, 2]),
        }
    }
}

impl SnowRenderer {
    pub fn render(&mut self, window: &WindowState) {
        {
            let fbuf = window.framebuf_size_u32();
            rg::begin_default_pass(&rg::PassAction::LOAD, fbuf[0], fbuf[1]);
        }
        self.shd.apply_pip();

        fn as_bytes<T>(x: &T) -> &[u8] {
            unsafe {
                std::slice::from_raw_parts(x as *const _ as *const _, std::mem::size_of::<T>())
            }
        }

        let size = glam::Vec2::from(window.size_f32());
        self.shd.set_fs_uniform(0, as_bytes(&size));

        let time = (Instant::now() - self.start_time).as_secs_f32();
        self.shd.set_fs_uniform(1, as_bytes(&time));

        // TODO: mouse position changes what?
        let mouse = glam::Vec2::from(window.size_f32()) / 2.0;
        self.shd.set_fs_uniform(2, as_bytes(&mouse));

        // just draw a fullscreen triangle
        self.mesh.draw_all();

        rg::end_pass();
    }
}
