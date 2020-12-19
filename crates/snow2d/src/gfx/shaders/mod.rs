//! Shaders

use rokol::gfx::{self as rg, BakedResource, Shader};

/// Creates a null-terminated string from static string
macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0");
    };
}

/// On release build, embed shader files
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

/// On debug build, dynamically load shader files
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
            block.size = std::mem::size_of::<glam::Mat4>() as i32;
            block.uniforms[0] = rg::ShaderUniformDesc {
                type_: rg::UniformType::Mat4 as u32,
                name: c_str!("transform").as_ptr() as *const _,
                ..Default::default()
            };
            block
        };
    })
}

pub fn aver() -> rokol::gfx::Shader {
    gen(&def_shd!("aver"), |desc| {
        desc.fs.images[0] = rg::ShaderImageDesc {
            type_: rg::ImageType::Dim2 as u32,
            name: c_str!("tex1").as_ptr() as *const _,
            ..Default::default()
        };

        desc.vs.uniform_blocks[0] = {
            let mut block = rg::ShaderUniformBlockDesc::default();
            block.size = std::mem::size_of::<glam::Mat4>() as i32;
            block.uniforms[0] = rg::ShaderUniformDesc {
                type_: rg::UniformType::Mat4 as u32,
                name: c_str!("transform").as_ptr() as *const _,
                ..Default::default()
            };
            block
        };
    })
}
