//! Shaders

#![allow(unused)]

use rokol::gfx::{self as rg, BakedResource};

use crate::gfx::Shader;

/// Creates a null-terminated string from static string
macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0");
    };
}

/// (Release build) embed shader files
///
/// [&str; 2]
#[cfg(not(debug_assertions))]
macro_rules! def_shd {
    ($file:expr) => {
        [
            concat!(include_str!(concat!("glsl/", $file, ".vs")), "\0").to_string(),
            concat!(include_str!(concat!("glsl/", $file, ".fs")), "\0").to_string(),
        ]
    };
}

/// (Debug build) dynamically load shader files
///
/// [String; 2]
#[cfg(debug_assertions)]
macro_rules! def_shd {
    ($file:expr) => {{
        use std::{fs, path::PathBuf};

        let dir = PathBuf::from(file!()).parent().unwrap().join("glsl");

        let mut vert = fs::read_to_string(dir.join($file).with_extension("vs")).unwrap();
        vert.push('\0');
        let mut frag = fs::read_to_string(dir.join($file).with_extension("fs")).unwrap();
        frag.push('\0');

        [vert, frag]
    }};
}

/// Creates vertex & fragment shader from files and modifies them with given procedure
fn gen(
    vs_fs: &[impl AsRef<str>; 2],
    mut_shd_desc: impl Fn(&mut rg::ShaderDesc),
    pip_desc: &mut rg::PipelineDesc,
) -> Shader {
    let mut shd_desc = unsafe { rokol::gfx::shader_desc(vs_fs[0].as_ref(), vs_fs[1].as_ref()) };
    mut_shd_desc(&mut shd_desc);
    let shd = rg::Shader::create(&shd_desc);

    pip_desc.shader = shd;
    let pip = rg::Pipeline::create(&pip_desc);

    Shader::new(shd, pip)
}

/// Position, color and uv
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct DefaultVertex {
    pub pos: [f32; 2],
    pub color: [u8; 4],
    pub uv: [f32; 2],
}

impl<Pos, Color, Uv> From<(Pos, Color, Uv)> for DefaultVertex
where
    Pos: Into<[f32; 2]>,
    Color: Into<[u8; 4]>,
    Uv: Into<[f32; 2]>,
{
    fn from(data: (Pos, Color, Uv)) -> Self {
        Self {
            pos: data.0.into(),
            color: data.1.into(),
            uv: data.2.into(),
        }
    }
}

impl DefaultVertex {
    pub fn layout_desc() -> rg::LayoutDesc {
        let mut desc = rg::LayoutDesc::default();
        desc.attrs[0].format = rg::VertexFormat::Float2 as u32;
        desc.attrs[1].format = rg::VertexFormat::UByte4N as u32;
        desc.attrs[2].format = rg::VertexFormat::Float2 as u32;
        desc
    }
}

const ALPHA_BLEND: rg::BlendState = rg::BlendState {
    enabled: true,
    src_factor_rgb: rg::BlendFactor::SrcAlpha as u32,
    dst_factor_rgb: rg::BlendFactor::OneMinusSrcAlpha as u32,
    op_rgb: 0,
    src_factor_alpha: rg::BlendFactor::One as u32,
    dst_factor_alpha: rg::BlendFactor::Zero as u32,
    op_alpha: 0,
};

pub fn default_screen() -> Shader {
    gen(
        &def_shd!("tex_1"),
        |desc| {
            desc.fs.images[0] = rg::ShaderImageDesc {
                image_type: rg::ImageType::Dim2 as u32,
                name: c_str!("tex1").as_ptr() as *const _,
                ..Default::default()
            };

            desc.vs.uniform_blocks[0] = {
                let mut block = rg::ShaderUniformBlockDesc::default();

                block.uniforms[0] = rg::ShaderUniformDesc {
                    type_: rg::UniformType::Mat4 as u32,
                    name: c_str!("transform").as_ptr() as *const _,
                    ..Default::default()
                };
                block.size += std::mem::size_of::<glam::Mat4>() as u64;

                block
            };
        },
        &mut {
            let mut desc = rg::PipelineDesc {
                index_type: rg::IndexType::UInt16 as u32,
                layout: DefaultVertex::layout_desc(),
                cull_mode: rg::CullMode::None as u32,
                ..Default::default()
            };
            desc.colors[0].blend = ALPHA_BLEND;
            desc
        },
    )
}

pub fn default_offscreen() -> Shader {
    gen(
        &def_shd!("tex_1"),
        |desc| {
            desc.fs.images[0] = rg::ShaderImageDesc {
                image_type: rg::ImageType::Dim2 as u32,
                name: c_str!("tex1").as_ptr() as *const _,
                ..Default::default()
            };

            desc.vs.uniform_blocks[0] = {
                let mut block = rg::ShaderUniformBlockDesc::default();

                block.uniforms[0] = rg::ShaderUniformDesc {
                    type_: rg::UniformType::Mat4 as u32,
                    name: c_str!("transform").as_ptr() as *const _,
                    ..Default::default()
                };
                block.size += std::mem::size_of::<glam::Mat4>() as u64;

                block
            };
        },
        &mut {
            let mut desc = rg::PipelineDesc {
                index_type: rg::IndexType::UInt16 as u32,
                layout: DefaultVertex::layout_desc(),
                cull_mode: rg::CullMode::None as u32,
                ..Default::default()
            };
            desc.depth.pixel_format = rg::PixelFormat::Depth as u32;
            desc
        },
    )
}

/// Tow-pass gaussian blur for ping pong frame buffers
///
/// <https://learnopengl.com/Advanced-Lighting/Bloom>
pub fn gauss() -> Shader {
    gen(
        &def_shd!("gauss"),
        |desc| {
            desc.fs.images[0] = rg::ShaderImageDesc {
                image_type: rg::ImageType::Dim2 as u32,
                name: c_str!("tex1").as_ptr() as *const _,
                ..Default::default()
            };

            desc.vs.uniform_blocks[0] = {
                let mut block = rg::ShaderUniformBlockDesc::default();

                block.uniforms[0] = rg::ShaderUniformDesc {
                    type_: rg::UniformType::Mat4 as u32,
                    name: c_str!("transform").as_ptr() as *const _,
                    ..Default::default()
                };
                block.size += std::mem::size_of::<glam::Mat4>() as u64;

                block
            };

            desc.vs.uniform_blocks[1] = {
                let mut block = rg::ShaderUniformBlockDesc::default();

                block.uniforms[0] = rg::ShaderUniformDesc {
                    type_: rg::UniformType::Float as u32,
                    name: c_str!("is_horizontal").as_ptr() as *const _,
                    ..Default::default()
                };
                block.size += std::mem::size_of::<f32>() as u64;

                block
            };
        },
        &mut {
            let mut desc = rg::PipelineDesc {
                index_type: rg::IndexType::UInt16 as u32,
                layout: DefaultVertex::layout_desc(),
                cull_mode: rg::CullMode::None as u32,
                ..Default::default()
            };
            desc.depth.pixel_format = rg::PixelFormat::Depth as u32;
            desc
        },
    )
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct PosUvVert {
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

pub fn snow() -> Shader {
    gen(
        &def_shd!("snow"),
        |desc| {
            desc.fs.uniform_blocks[0] = {
                let mut block = rg::ShaderUniformBlockDesc::default();

                block.uniforms[0] = rg::ShaderUniformDesc {
                    type_: rg::UniformType::Float2 as u32,
                    name: c_str!("iResolution").as_ptr() as *const _,
                    ..Default::default()
                };
                block.size += std::mem::size_of::<[f32; 2]>() as u64;

                block
            };

            desc.fs.uniform_blocks[1] = {
                let mut block = rg::ShaderUniformBlockDesc::default();

                block.uniforms[0] = rg::ShaderUniformDesc {
                    type_: rg::UniformType::Float as u32,
                    name: c_str!("iTime").as_ptr() as *const _,
                    ..Default::default()
                };
                block.size += std::mem::size_of::<f32>() as u64;

                block
            };

            desc.fs.uniform_blocks[2] = {
                let mut block = rg::ShaderUniformBlockDesc::default();

                block.uniforms[0] = rg::ShaderUniformDesc {
                    type_: rg::UniformType::Float2 as u32,
                    name: c_str!("iMouse").as_ptr() as *const _,
                    ..Default::default()
                };
                block.size += std::mem::size_of::<[f32; 2]>() as u64;

                block
            };
        },
        &mut {
            let mut desc = rg::PipelineDesc {
                index_type: rg::IndexType::UInt16 as u32,
                layout: PosUvVert::layout_desc(),
                cull_mode: rg::CullMode::None as u32,
                ..Default::default()
            };
            desc.colors[0].blend = ALPHA_BLEND;
            desc
        },
    )
}
