use {
    image::GenericImageView,
    rokol::gfx::{self as rg, BakedResource},
    std::path::Path,
};

use crate::gfx::batcher::draw::{OnSpritePush, QuadParamsBuilder, Texture2d};

fn gen_img(pixels: &[u8], w: u32, h: u32) -> rg::Image {
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

// TODO: drop type and shared type
#[derive(Debug, Clone)]
pub struct TextureData2d {
    pub img: rg::Image,
    pub w: u32,
    pub h: u32,
}

impl Default for TextureData2d {
    fn default() -> Self {
        Self {
            img: rg::Image { id: u32::MAX },
            w: 0,
            h: 0,
        }
    }
}

impl TextureData2d {
    pub fn from_path(path: impl AsRef<Path>) -> image::ImageResult<Self> {
        let img = image::open(path)?;

        // TODO: flip?
        // #[cfg(rokol_gfx = "glcore33")]
        // let img = img.flipv();

        Ok(Self::from_pixels(img.as_bytes(), img.width(), img.height()))
    }

    pub fn from_encoded_bytes(bytes: &[u8]) -> image::ImageResult<Self> {
        let img = image::load_from_memory(bytes)?;

        // TODO: flip?
        // #[cfg(rokol_gfx = "glcore33")]
        // let img = img.flipv();

        Ok(Self::from_pixels(img.as_bytes(), img.width(), img.height()))
    }

    fn from_dyn_image(img: image::DynamicImage) -> Self {
        Self::from_pixels(img.as_bytes(), img.width(), img.height())
    }

    // TODO: flip?
    pub fn from_pixels(pixels: &[u8], w: u32, h: u32) -> Self {
        let id = self::gen_img(pixels, w, h);
        Self { img: id, w, h }
    }
}

impl Texture2d for TextureData2d {
    fn raw_texture(&self) -> rg::Image {
        self.img
    }

    fn w(&self) -> f32 {
        self.w as f32
    }

    fn h(&self) -> f32 {
        self.h as f32
    }
}

impl OnSpritePush for TextureData2d {
    fn to_texture(&self) -> TextureData2d {
        self.clone()
    }

    fn on_sprite_push(&self, builder: &mut impl QuadParamsBuilder) {
        builder.src_rect_px([0.0, 0.0, self.w(), self.h()]);
    }
}
