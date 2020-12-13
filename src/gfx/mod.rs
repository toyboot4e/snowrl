//! Graphics

pub mod batch;
pub mod shaders;

use {
    image::{io::Reader as ImageReader, GenericImageView},
    rokol::gfx::{self as rg, BakedResource},
    std::path::Path,
};

pub fn load_img(path: &Path) -> rg::Image {
    let img = ImageReader::open(path).unwrap().decode().unwrap();

    // #[cfg(rokol_gfx = "glcore33")]
    let img = img.flipv();

    let (w, h) = img.dimensions();
    let pixels = img.as_bytes();

    rg::Image::create(&{
        let mut desc = rg::ImageDesc {
            type_: rg::ImageType::Dim2 as u32,
            width: w as i32,
            height: h as i32,
            usage: rg::ResourceUsage::Immutable as u32,
            min_filter: rg::Filter::Linear as u32,
            mag_filter: rg::Filter::Linear as u32,
            ..Default::default()
        };

        desc.content.subimage[0][0] = rg::SubimageContent {
            ptr: pixels.as_ptr() as *const _,
            size: pixels.len() as i32,
        };

        desc
    })
}
