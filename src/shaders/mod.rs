//! Shaders

use rokol::gfx::{self as rg, BakedResource, Shader};

/// Creates a null-terminated string from static string
macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0");
    };
}

macro_rules! def_shd {
    ($file:expr) => {
        [
            concat!(include_str!(concat!("glsl/", $file, ".vert")), "\0"),
            concat!(include_str!(concat!("glsl/", $file, ".frag")), "\0"),
        ];
    };
}

fn gen(vs_fs: &[&str; 2], f: impl Fn(&mut rg::ShaderDesc)) -> rg::Shader {
    let mut desc = unsafe { rokol::gfx::shader_desc(vs_fs[0], vs_fs[1]) };
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
    })
}

pub fn tex_2() -> rokol::gfx::Shader {
    gen(&def_shd!("tex_2"), |desc| {
        desc.fs.images[0] = rg::ShaderImageDesc {
            type_: rg::ImageType::Dim2 as u32,
            name: c_str!("tex1").as_ptr() as *const _,
            ..Default::default()
        };
        desc.fs.images[1] = rg::ShaderImageDesc {
            type_: rg::ImageType::Dim2 as u32,
            name: c_str!("tex2").as_ptr() as *const _,
            ..Default::default()
        };
    })
}

