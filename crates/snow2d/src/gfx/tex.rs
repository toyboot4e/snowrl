/*!

2D texture types

*/

// TODO: texture builder

use {
    image::GenericImageView,
    rokol::gfx::{self as rg, BakedResource},
    std::path::Path,
};

use crate::{
    asset::{self, Asset, AssetItem, AssetLoader},
    gfx::{
        draw::{DrawApiData, OnSpritePush, QuadIter, QuadParamsBuilder, Texture2d},
        geom2d::{Flips, Scaled, Vec2f},
        Color,
    },
};

/// Image loading result
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

        desc.data.subimage[0][0] = pixels.into();
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

/// A 2D owned texture that frees GPU image on drop. It's an [`AssetItem`].
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
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let img = image::open(path)?;

        Ok(Self::from_pixels(img.as_bytes(), img.width(), img.height()))
    }

    pub fn from_encoded_bytes(bytes: &[u8]) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;

        Ok(Self::from_pixels(img.as_bytes(), img.width(), img.height()))
    }

    /// The width and height have to be in scaled size (e.g. if on 2x DPI monitor with 1280x720 scaled
    /// screen size, pass 1280x720)
    pub fn from_pixels(pixels: &[u8], w: u32, h: u32) -> Self {
        let id = self::gen_img(pixels, w, h);
        Self { img: id, w, h }
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

impl AssetItem for Texture2dDrop {
    type Loader = TextureLoader;
}

/// [`AssetLoader`] for [`Texture2dDrop`]
#[derive(Debug)]
pub struct TextureLoader;

impl AssetLoader for TextureLoader {
    type Item = Texture2dDrop;
    fn load(&mut self, path: &Path) -> asset::Result<Self::Item> {
        use std::io::{Error, ErrorKind};
        Texture2dDrop::from_path(path).map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

/// Texture with uv values
#[derive(Debug, Clone)]
pub struct SharedSubTexture2d {
    pub tex: Asset<Texture2dDrop>,
    /// [x, y, width, height]
    pub uv_rect: [f32; 4],
}

/// Full-featured texture
#[derive(Debug, Clone)]
pub struct SpriteData {
    pub tex: Asset<Texture2dDrop>,
    /// [x, y, width, height]
    pub uv_rect: [f32; 4],
    pub rot: f32,
    pub origin: [f32; 2],
    pub scales: [f32; 2],
    pub color: Color,
}

impl Default for SpriteData {
    fn default() -> Self {
        Self {
            tex: Asset::empty(),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            rot: 0.0,
            // left-up corner
            origin: [0.0, 0.0],
            scales: [1.0, 1.0],
            color: Color::WHITE,
        }
    }
}

/// A nine slice texture is suitable for e.g., window image
#[derive(Debug, Clone)]
pub struct NineSliceSprite {
    pub tex: Asset<Texture2dDrop>,
}

// --------------------------------------------------------------------------------
// Trait implementations

// ----------------------------------------

impl Texture2d for Texture2dDrop {
    fn img(&self) -> rg::Image {
        self.img
    }

    fn sub_tex_size_scaled(&self) -> [f32; 2] {
        [self.w as f32, self.h as f32]
    }

    fn sub_tex_size_unscaled(&self) -> [f32; 2] {
        self.sub_tex_size_scaled()
    }
}

impl Texture2d for SharedSubTexture2d {
    fn img(&self) -> rg::Image {
        if let Some(tex) = self.tex.get() {
            tex.img()
        } else {
            // TODO: is this OK?
            Default::default()
        }
    }

    fn sub_tex_size_unscaled(&self) -> [f32; 2] {
        if let Some(tex) = self.tex.get() {
            let size = tex.sub_tex_size_unscaled();
            [size[0] * self.uv_rect[2], size[1] * self.uv_rect[3]]
        } else {
            // TODO: is this OK?
            [0.0, 0.0]
        }
    }

    fn sub_tex_size_scaled(&self) -> [f32; 2] {
        self.sub_tex_size_unscaled()
    }
}

impl Texture2d for SpriteData {
    fn img(&self) -> rg::Image {
        // TODO: don't lock?
        if let Some(tex) = self.tex.get() {
            tex.img()
        } else {
            // TODO: is this OK?
            Default::default()
        }
    }

    fn sub_tex_size_unscaled(&self) -> [f32; 2] {
        // TODO: don't lock?
        if let Some(tex) = self.tex.get() {
            let size = tex.sub_tex_size_unscaled();
            [size[0] * self.uv_rect[2], size[1] * self.uv_rect[3]]
        } else {
            // TODO: is this OK?
            [0.0, 0.0]
        }
    }

    fn sub_tex_size_scaled(&self) -> [f32; 2] {
        let size = self.sub_tex_size_unscaled();
        [size[0] * self.scales[0], size[1] * self.scales[1]]
    }
}

impl Texture2d for NineSliceSprite {
    // TODO: don't lock?
    fn img(&self) -> rg::Image {
        if let Some(tex) = self.tex.get() {
            tex.img()
        } else {
            // TODO: is this OK?
            Default::default()
        }
    }

    fn sub_tex_size_unscaled(&self) -> [f32; 2] {
        // TODO: don't lock?
        if let Some(tex) = self.tex.get() {
            tex.sub_tex_size_unscaled()
        } else {
            // TODO: is this OK?
            [0.0, 0.0]
        }
    }

    fn sub_tex_size_scaled(&self) -> [f32; 2] {
        self.sub_tex_size_unscaled()
    }
}

// ----------------------------------------

impl OnSpritePush for Texture2dDrop {
    fn init_quad(&self, builder: &mut impl QuadParamsBuilder) {
        let size = self.sub_tex_size_unscaled();
        builder
            .src_rect_px(([0.0, 0.0], size))
            .dst_size_px(size)
            .uv_rect([0.0, 0.0, 1.0, 1.0]);
    }
}

impl OnSpritePush for SharedSubTexture2d {
    fn init_quad(&self, builder: &mut impl QuadParamsBuilder) {
        let size = self.sub_tex_size_unscaled();
        builder
            .src_rect_px(([0.0, 0.0], size))
            .dst_size_px(size)
            .uv_rect(self.uv_rect);
    }
}

impl OnSpritePush for SpriteData {
    fn init_quad(&self, builder: &mut impl QuadParamsBuilder) {
        let size = self.sub_tex_size_unscaled();
        builder
            .src_rect_px(([0.0, 0.0], size))
            .dst_size_px([size[0] * self.scales[0], size[1] * self.scales[1]])
            .uv_rect(self.uv_rect)
            .rot(self.rot)
            .origin(self.origin)
            .color(self.color);
    }
}

impl OnSpritePush for NineSliceSprite {
    fn init_quad(&self, builder: &mut impl QuadParamsBuilder) {
        if let Some(tex) = self.tex.get() {
            tex.init_quad(builder);
        }
    }

    #[inline]
    fn push_quad<Q: QuadIter>(&self, draw: &mut DrawApiData<Q>, flips: Flips) {
        let tex = match self.tex.get() {
            Some(tex) => tex,
            None => {
                // FIXME: call default implementation
                draw.params.write_to_quad(
                    draw.quad_iter.next_quad_mut(Default::default()),
                    self,
                    flips,
                );
                return;
            }
        };

        let (dst_pos, dst_size) = match &draw.params.dst_rect {
            Scaled::Normalized(_rect) => unimplemented!(),
            Scaled::Px(rect) => ([rect.x, rect.y], [rect.w, rect.h]),
        };

        let size = tex.sub_tex_size_unscaled();
        let ws = {
            let w = size[0] / 3.0;
            [w, dst_size[0] - 2.0 * w, w]
        };

        let hs = {
            let h = size[1] / 3.0;
            [h, dst_size[1] - 2.0 * h, h]
        };

        for i in 0..9 {
            let ix = i % 3;
            let iy = i / 3;

            let uv = [ix as f32 / 3.0, iy as f32 / 3.0, 1.0 / 3.0, 1.0 / 3.0];

            let pos = Vec2f::new(
                dst_pos[0] + ws.iter().take(ix).sum::<f32>(),
                dst_pos[1] + hs.iter().take(iy).sum::<f32>(),
            );

            draw.params
                .uv_rect(uv)
                .dst_pos_px(pos)
                .dst_size_px([ws[ix], hs[iy]]);

            use std::ops::Deref;
            draw.params
                .write_to_quad(draw.quad_iter.next_quad_mut(tex.img()), tex.deref(), flips);
        }
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

    /// [`rokol::gfx::Pass`] for off-screen rendering
    pub fn pass(&self) -> rg::Pass {
        self.pass
    }

    pub fn tex(&self) -> &Texture2dDrop {
        &self.tex
    }

    pub fn img(&self) -> rg::Image {
        self.tex.img()
    }
}
