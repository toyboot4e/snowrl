use {
    image::GenericImageView,
    rokol::gfx::{self as rg, BakedResource},
    std::{path::Path, rc::Rc},
};

use crate::gfx::batcher::draw::{CheatTexture2d, QuadParamsBuilder};
use crate::gfx::batcher::draw::{OnSpritePush, Sprite, SubTexture2d, Texture2d};

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

/// Frees GPU image on drop
#[derive(Debug, Default)]
pub struct Texture2dDrop {
    img: rg::Image,
    w: u32,
    h: u32,
}

impl Drop for Texture2dDrop {
    fn drop(&mut self) {
        rg::Image::destroy(self.img);
    }
}

impl Texture2dDrop {
    pub fn from_path(path: impl AsRef<Path>) -> image::ImageResult<Self> {
        let img = image::open(path)?;

        Ok(Self::from_pixels(img.as_bytes(), img.width(), img.height()))
    }

    pub fn from_encoded_bytes(bytes: &[u8]) -> image::ImageResult<Self> {
        let img = image::load_from_memory(bytes)?;

        Ok(Self::from_pixels(img.as_bytes(), img.width(), img.height()))
    }

    pub fn from_pixels(pixels: &[u8], w: u32, h: u32) -> Self {
        let id = self::gen_img(pixels, w, h);
        Self { img: id, w, h }
    }

    pub fn into_shared(self) -> SharedTexture2d {
        SharedTexture2d { tex: Rc::new(self) }
    }
}

/// Reference counted version of [`Texture2dDrop`]
#[derive(Debug, Clone)]
pub struct SharedTexture2d {
    pub tex: Rc<Texture2dDrop>,
}

#[derive(Debug, Clone)]
pub struct SharedSubTexture2d {
    pub shared: SharedTexture2d,
    /// x, y, w, h
    pub uv_rect: [f32; 4],
}

// --------------------------------------------------------------------------------
// Trait implementations

// ----------------------------------------

impl AsRef<Texture2dDrop> for Texture2dDrop {
    fn as_ref(&self) -> &Texture2dDrop {
        self
    }
}

impl AsRef<Texture2dDrop> for SharedTexture2d {
    fn as_ref(&self) -> &Texture2dDrop {
        &self.tex
    }
}

impl AsRef<Texture2dDrop> for SharedSubTexture2d {
    fn as_ref(&self) -> &Texture2dDrop {
        &self.shared.tex
    }
}

// ----------------------------------------

impl<T: AsRef<Texture2dDrop>> Texture2d for T {
    fn img(&self) -> rg::Image {
        self.as_ref().img
    }

    fn w(&self) -> f32 {
        self.as_ref().w as f32
    }

    fn h(&self) -> f32 {
        self.as_ref().h as f32
    }
}

// ----------------------------------------

impl OnSpritePush for Texture2dDrop {
    fn to_cheat_texture(&self) -> CheatTexture2d {
        CheatTexture2d {
            img: self.as_ref().img,
            w: self.as_ref().w,
            h: self.as_ref().h,
        }
    }

    fn on_sprite_push(&self, builder: &mut impl QuadParamsBuilder) {
        builder
            .src_rect_px([0.0, 0.0, self.as_ref().w(), self.as_ref().h()])
            .dst_size_px([self.as_ref().w(), self.as_ref().h()]);
    }
}

impl OnSpritePush for SharedTexture2d {
    fn to_cheat_texture(&self) -> CheatTexture2d {
        self.as_ref().to_cheat_texture()
    }

    fn on_sprite_push(&self, builder: &mut impl QuadParamsBuilder) {
        self.as_ref().on_sprite_push(builder);
    }
}

impl OnSpritePush for SharedSubTexture2d {
    fn to_cheat_texture(&self) -> CheatTexture2d {
        self.as_ref().to_cheat_texture()
    }

    fn on_sprite_push(&self, builder: &mut impl QuadParamsBuilder) {
        builder
            .src_rect_px([0.0, 0.0, self.as_ref().w(), self.as_ref().h()])
            .dst_size_px([self.as_ref().w(), self.as_ref().h()])
            .uv_rect(self.uv_rect);
    }
}
