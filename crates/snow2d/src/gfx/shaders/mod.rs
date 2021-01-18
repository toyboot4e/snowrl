//! Shaders

use rokol::gfx::{self as rg, BakedResource, Shader};

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
fn gen(vs_fs: &[impl AsRef<str>; 2], f: impl Fn(&mut rg::ShaderDesc)) -> rg::Shader {
    let mut desc = unsafe { rokol::gfx::shader_desc(vs_fs[0].as_ref(), vs_fs[1].as_ref()) };
    f(&mut desc);
    Shader::create(&desc)
}

pub fn tex_1() -> rokol::gfx::Shader {
    gen(&def_shd!("tex_1"), |desc| {
        desc.fs.images[0] = rg::ShaderImageDesc {
            type_: rg::ImageType::Dim2 as u32,
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
            block.size += std::mem::size_of::<glam::Mat4>() as i32;

            block
        };
    })
}

/// Tow-pass gaussian blur for ping pong frame buffers
///
/// <https://learnopengl.com/Advanced-Lighting/Bloom>
pub fn gauss() -> rokol::gfx::Shader {
    gen(&def_shd!("gauss"), |desc| {
        desc.fs.images[0] = rg::ShaderImageDesc {
            type_: rg::ImageType::Dim2 as u32,
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
            block.size += std::mem::size_of::<glam::Mat4>() as i32;

            block
        };

        desc.vs.uniform_blocks[1] = {
            let mut block = rg::ShaderUniformBlockDesc::default();

            block.uniforms[0] = rg::ShaderUniformDesc {
                type_: rg::UniformType::Float as u32,
                name: c_str!("is_horizontal").as_ptr() as *const _,
                ..Default::default()
            };
            block.size += std::mem::size_of::<f32>() as i32;

            block
        };
    })
}

pub fn snow() -> rokol::gfx::Shader {
    gen(&def_shd!("snow"), |desc| {
        desc.vs.uniform_blocks[0] = {
            let mut block = rg::ShaderUniformBlockDesc::default();

            block.uniforms[0] = rg::ShaderUniformDesc {
                type_: rg::UniformType::Mat4 as u32,
                name: c_str!("transform").as_ptr() as *const _,
                ..Default::default()
            };
            block.size += std::mem::size_of::<glam::Mat4>() as i32;

            block
        };

        desc.fs.uniform_blocks[0] = {
            let mut block = rg::ShaderUniformBlockDesc::default();

            block.uniforms[0] = rg::ShaderUniformDesc {
                type_: rg::UniformType::Float2 as u32,
                name: c_str!("iResolution").as_ptr() as *const _,
                ..Default::default()
            };
            block.size += std::mem::size_of::<[f32; 2]>() as i32;

            block
        };

        desc.fs.uniform_blocks[1] = {
            let mut block = rg::ShaderUniformBlockDesc::default();

            block.uniforms[0] = rg::ShaderUniformDesc {
                type_: rg::UniformType::Float as u32,
                name: c_str!("iTime").as_ptr() as *const _,
                ..Default::default()
            };
            block.size += std::mem::size_of::<f32>() as i32;

            block
        };

        desc.fs.uniform_blocks[2] = {
            let mut block = rg::ShaderUniformBlockDesc::default();

            block.uniforms[0] = rg::ShaderUniformDesc {
                type_: rg::UniformType::Float2 as u32,
                name: c_str!("iMouse").as_ptr() as *const _,
                ..Default::default()
            };
            block.size += std::mem::size_of::<[f32; 2]>() as i32;

            block
        };
    })
}
