//! 2D texture types

// TODO: do not use composition

use {
    image::GenericImageView,
    rokol::gfx::{self as rg, BakedResource},
    std::{path::Path, rc::Rc},
};

use crate::gfx::{
    batcher::draw::{
        CheatTexture2d, DrawApiData, OnSpritePush, QuadIter, QuadParamsBuilder, Texture2d,
    },
    geom2d::Flips,
};

pub type Result<T> = image::ImageResult<T>;

fn gen_img(pixels: &[u8], w: u32, h: u32) -> rg::Image {
    rg::Image::create(&{
        let mut desc = rg::ImageDesc {
            type_: rg::ImageType::Dim2 as u32,
            render_target: false,
            width: w as i32,
            height: h as i32,
            usage: rg::ResourceUsage::Immutable as u32,
            min_filter: rg::Filter::Nearest as u32,
            mag_filter: rg::Filter::Nearest as u32,
            ..Default::default()
        };

        desc.content.subimage[0][0] = rg::SubImageContent {
            ptr: pixels.as_ptr() as *const _,
            size: pixels.len() as i32,
        };

        desc
    })
}

fn target_desc(w: u32, h: u32) -> rg::ImageDesc {
    rg::ImageDesc {
        type_: rg::ImageType::Dim2 as u32,
        render_target: true,
        width: w as i32,
        height: h as i32,
        // usage: rg::ResourceUsage::Immutable as u32,
        min_filter: rg::Filter::Linear as u32,
        mag_filter: rg::Filter::Linear as u32,
        // TODO: (see also: rasterizer in pipeline)
        wrap_u: rg::Wrap::ClampToEdge as u32,
        wrap_v: rg::Wrap::ClampToEdge as u32,
        wrap_w: rg::Wrap::ClampToEdge as u32,
        sample_count: 1,
        ..Default::default()
    }
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

/// The width and height have to be in scaled size (e.g. if on 2x DPI monitor with 1280x720 scaled
/// screen size, pass 1280x720)
impl Texture2dDrop {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let img = image::open(path)?;

        Ok(Self::from_pixels(img.as_bytes(), img.width(), img.height()))
    }

    pub fn from_encoded_bytes(bytes: &[u8]) -> Result<Self> {
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

    fn offscreen(w: u32, h: u32) -> (Self, rg::ImageDesc) {
        let desc = self::target_desc(w, h);
        let me = Self {
            img: rg::Image::create(&desc),
            w,
            h,
        };
        (me, desc)
    }
}

/// Reference counted version of [`Texture2dDrop`]
#[derive(Debug, Clone)]
pub struct SharedTexture2d {
    pub tex: Rc<Texture2dDrop>,
}

impl SharedTexture2d {
    /// uv_rect: [x, y, width, height]
    pub fn split(&self, uv_rect: impl Into<[f32; 4]>) -> SharedSubTexture2d {
        SharedSubTexture2d {
            shared: self.clone(),
            uv_rect: uv_rect.into(),
        }
    }
}

/// [`SharedTexture2d`] with uv rectangle
#[derive(Debug, Clone)]
pub struct SharedSubTexture2d {
    pub shared: SharedTexture2d,
    /// [x, y, width, height]
    pub uv_rect: [f32; 4],
}

/// Full-featured reference counted sub texture
#[derive(Debug, Clone)]
pub struct SpriteData {
    pub sub_tex: SharedSubTexture2d,
    pub rot: f32,
    pub origin: [f32; 2],
    pub scale: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct NineSliceSprite {
    pub sprite: SpriteData,
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

impl AsRef<Texture2dDrop> for SpriteData {
    fn as_ref(&self) -> &Texture2dDrop {
        &self.sub_tex.shared.tex
    }
}

impl AsRef<Texture2dDrop> for NineSliceSprite {
    fn as_ref(&self) -> &Texture2dDrop {
        &self.sprite.sub_tex.shared.tex
    }
}

// ----------------------------------------

impl Texture2d for Texture2dDrop {
    fn img(&self) -> rg::Image {
        self.img
    }

    fn w(&self) -> f32 {
        self.w as f32
    }

    fn h(&self) -> f32 {
        self.h as f32
    }
}

impl Texture2d for SharedTexture2d {
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

impl Texture2d for SharedSubTexture2d {
    fn img(&self) -> rg::Image {
        self.as_ref().img
    }

    fn w(&self) -> f32 {
        self.as_ref().w as f32 * self.uv_rect[2] as f32
    }

    fn h(&self) -> f32 {
        self.as_ref().h as f32 * self.uv_rect[3] as f32
    }
}

impl Texture2d for SpriteData {
    fn img(&self) -> rg::Image {
        self.sub_tex.img()
    }

    fn w(&self) -> f32 {
        self.sub_tex.w()
    }

    fn h(&self) -> f32 {
        self.sub_tex.h()
    }
}

impl Texture2d for NineSliceSprite {
    fn img(&self) -> rg::Image {
        self.sprite.img()
    }

    fn w(&self) -> f32 {
        self.sprite.w()
    }

    fn h(&self) -> f32 {
        self.sprite.h()
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

    fn init_quad(&self, builder: &mut impl QuadParamsBuilder) {
        builder
            .src_rect_px([0.0, 0.0, self.as_ref().w(), self.as_ref().h()])
            .dst_size_px([self.as_ref().w(), self.as_ref().h()])
            .uv_rect([0.0, 0.0, 1.0, 1.0]);
    }
}

impl OnSpritePush for SharedTexture2d {
    fn to_cheat_texture(&self) -> CheatTexture2d {
        self.as_ref().to_cheat_texture()
    }

    fn init_quad(&self, builder: &mut impl QuadParamsBuilder) {
        self.as_ref().init_quad(builder);
    }
}

impl OnSpritePush for SharedSubTexture2d {
    fn to_cheat_texture(&self) -> CheatTexture2d {
        self.as_ref().to_cheat_texture()
    }

    fn init_quad(&self, builder: &mut impl QuadParamsBuilder) {
        builder
            .src_rect_px([0.0, 0.0, self.as_ref().w(), self.as_ref().h()])
            .dst_size_px([self.w(), self.h()])
            .uv_rect(self.uv_rect);
    }
}

impl OnSpritePush for SpriteData {
    fn to_cheat_texture(&self) -> CheatTexture2d {
        self.as_ref().to_cheat_texture()
    }

    fn init_quad(&self, builder: &mut impl QuadParamsBuilder) {
        builder
            .src_rect_px([0.0, 0.0, self.w(), self.h()])
            .dst_size_px([self.w() * self.scale[0], self.h() * self.scale[1]])
            .uv_rect(self.sub_tex.uv_rect)
            .rot(self.rot)
            .origin(self.origin);
    }
}

impl OnSpritePush for NineSliceSprite {
    fn to_cheat_texture(&self) -> CheatTexture2d {
        self.sprite.to_cheat_texture()
    }

    fn init_quad(&self, builder: &mut impl QuadParamsBuilder) {
        self.sprite.init_quad(builder);
    }

    #[inline]
    fn push_quad<Q: QuadIter>(&self, draw: &mut DrawApiData<Q>, flips: Flips) {
        // TODO: nince slice rendering
        let tex = self.to_cheat_texture();
        draw.params
            .write_to_quad(draw.quad_iter.next_quad_mut(tex.img), &tex, flips);
    }
}

// --------------------------------------------------------------------------------

/// Off-screen rendering target
#[derive(Debug, Default)]
pub struct RenderTexture {
    /// Render target texture binded to the internal [`rg::Pass`]
    tex: Texture2dDrop,
    /// Off-screen rendering pass
    pass: rg::Pass,
}

impl RenderTexture {
    /// The width and height have to be in scaled size (e.g. if on 2x DPI monitor with 1280x720
    /// scaled screen size, pass 1280x720)
    pub fn new(w: u32, h: u32) -> Self {
        let (tex, mut image_desc) = Texture2dDrop::offscreen(w, h);

        let pass = rg::Pass::create(&{
            let mut desc = rg::PassDesc::default();

            // color image
            desc.color_attachments[0].image = tex.img();

            // depth image
            desc.depth_stencil_attachment.image = rg::Image::create(&{
                image_desc.pixel_format = rg::PixelFormat::Depth as u32;
                image_desc
            });

            desc
        });

        Self { tex, pass }
    }

    pub fn tex(&self) -> &Texture2dDrop {
        &self.tex
    }

    pub fn pass(&self) -> rg::Pass {
        self.pass
    }

    pub fn img(&self) -> rg::Image {
        self.tex.img()
    }
}
