/*!
2D texture types

They implement traits in the [`draw`] module.

[`draw`]: super::draw
*/

// TODO: texture builder

use {
    image::GenericImageView,
    rokol::gfx::{self as rg, BakedResource},
    serde::{Deserialize, Serialize},
    std::path::Path,
};

use crate::{
    self as snow2d,
    asset::{self, Asset, AssetItem, AssetLoader},
    gfx::{
        draw::{DrawApiData, OnSpritePush, QuadIter, QuadParamsBuilder, Texture2d},
        geom2d::{Flips, Scaled, Vec2f},
        Color,
    },
    utils::Inspect,
};

/// Image loading result
pub type Result<T> = image::ImageResult<T>;

#[derive(Debug)]
pub struct TextureBuilder {
    // TODO: Cow
    pixels: Vec<u8>,
    size: [u32; 2],
    filter: rg::Filter,
    wrap: rg::Wrap,
}

impl TextureBuilder {
    pub fn from_path(path: &Path) -> Result<Self> {
        Ok(Self::from_dyn_img(image::open(path)?))
    }

    pub fn from_encoded_bytes(mem: &[u8]) -> Result<Self> {
        Ok(Self::from_dyn_img(image::load_from_memory(mem)?))
    }

    fn from_dyn_img(img: image::DynamicImage) -> Self {
        let size = [img.width(), img.height()];

        // NOTE: We have to make sure it's RGBA8. It can be, for example, RGB8 image.
        let img: Vec<u8> = img.into_rgba8().into_raw();

        Self {
            pixels: img,
            size,
            filter: rg::Filter::Nearest,
            wrap: rg::Wrap::ClampToEdge,
        }
    }
}

impl TextureBuilder {
    pub fn from_pixels(pixels: &[u8], w: u32, h: u32) -> Self {
        Self {
            pixels: Vec::from(pixels),
            size: [w, h],
            filter: rg::Filter::Nearest,
            wrap: rg::Wrap::ClampToEdge,
        }
    }

    pub fn filter(&mut self, filter: rg::Filter) -> &mut Self {
        self.filter = filter;
        self
    }

    pub fn wrap(&mut self, wrap: rg::Wrap) -> &mut Self {
        self.wrap = wrap;
        self
    }

    pub fn build_texture(&self) -> Texture2dDrop {
        Texture2dDrop {
            img: rg::Image::create(&{
                let mut desc = self::img_desc(self.size[0], self.size[1], self.filter, self.wrap);
                desc.render_target = false;
                desc.usage = rg::ResourceUsage::Immutable as u32;
                desc.data.subimage[0][0] = self.pixels.as_slice().into();
                desc
            }),
            w: self.size[0],
            h: self.size[1],
        }
    }
}

/// Set usage and pixels or depth format
fn img_desc(w: u32, h: u32, filter: rg::Filter, wrap: rg::Wrap) -> rg::ImageDesc {
    rg::ImageDesc {
        type_: rg::ImageType::Dim2 as u32,
        width: w as i32,
        height: h as i32,
        min_filter: filter as u32,
        mag_filter: filter as u32,
        wrap_u: wrap as u32,
        wrap_v: wrap as u32,
        wrap_w: wrap as u32,
        ..Default::default()
    }
}

/// Owned 2D texture
///
/// Frees GPU image on drop. It's an [`AssetItem`].
#[derive(Debug, PartialEq, Default)]
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
        Ok(TextureBuilder::from_path(path)
            .map_err(|e| Error::new(ErrorKind::Other, e))?
            .build_texture())
    }
}

/// Texture with uv values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedSubTexture2d {
    pub tex: Asset<Texture2dDrop>,
    /// [x, y, width, height]
    pub uv_rect: [f32; 4],
}

/// Full-featured texture
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Inspect)]
pub struct SpriteData {
    pub tex: Asset<Texture2dDrop>,
    /// [x, y, width, height]
    pub uv_rect: [f32; 4],
    pub rot: f32,
    pub origin: [f32; 2],
    pub scales: [f32; 2],
    pub color: Color,
}

impl SpriteData {
    pub fn from_tex(tex: Asset<Texture2dDrop>) -> Self {
        Self {
            tex,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            rot: 0.0,
            origin: [0.0, 0.0],
            scales: [1.0, 1.0],
            color: Color::WHITE,
        }
    }

    pub fn builder(tex: Asset<Texture2dDrop>) -> SpriteBuilder {
        SpriteBuilder {
            sprite: Self::from_tex(tex),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpriteBuilder {
    pub sprite: SpriteData,
}

impl SpriteBuilder {
    pub fn build(&self) -> SpriteData {
        self.sprite.clone()
    }

    pub fn uv_rect(&mut self, uv_rect: [f32; 4]) -> &mut Self {
        self.sprite.uv_rect = uv_rect;
        self
    }

    pub fn rot(&mut self, rot: f32) -> &mut Self {
        self.sprite.rot = rot;
        self
    }

    pub fn origin(&mut self, origin: [f32; 2]) -> &mut Self {
        self.sprite.origin = origin;
        self
    }

    pub fn scales(&mut self, scales: [f32; 2]) -> &mut Self {
        self.sprite.scales = scales;
        self
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.sprite.color = color;
        self
    }
}

/// Suitable for, e.g., window image
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Inspect)]
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
            tex.img
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
            tex.img
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
            tex.img
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

#[derive(Debug)]
pub struct RenderTextureBuilder {
    size: [u32; 2],
    filter: rg::Filter,
    wrap: rg::Wrap,
}

impl RenderTextureBuilder {
    /// Set scaled size
    pub fn begin(size: [u32; 2]) -> Self {
        Self {
            size,
            filter: rg::Filter::Linear,
            wrap: rg::Wrap::ClampToEdge,
        }
    }

    pub fn filter(&mut self, filter: rg::Filter) -> &mut Self {
        self.filter = filter;
        self
    }

    pub fn wrap(&mut self, wrap: rg::Wrap) -> &mut Self {
        self.wrap = wrap;
        self
    }

    pub fn build(&self) -> RenderTexture {
        let mut img_desc = self::img_desc(self.size[0], self.size[1], self.filter, self.wrap);
        img_desc.render_target = true;
        // render target has to have `Immutable` usage
        img_desc.usage = rg::ResourceUsage::Immutable as u32;

        let tex = Texture2dDrop {
            img: rg::Image::create(&img_desc),
            w: self.size[0],
            h: self.size[1],
        };

        let pass = rg::Pass::create(&{
            let mut pass_desc = rg::PassDesc::default();

            // color image
            pass_desc.color_attachments[0].image = tex.img;

            // depth image
            // TODO: can we skip this modifying shader creation
            pass_desc.depth_stencil_attachment.image = rg::Image::create(&rg::ImageDesc {
                // NOTE: Both `PipelineDesc` and off-screen render target `ImageDesc`
                //       must have `pixel_format` being `Depth`.
                pixel_format: rg::PixelFormat::Depth as u32,
                ..img_desc
            });

            pass_desc
        });

        RenderTexture { tex, pass }
    }
}

/// Off-screen rendering target
#[derive(Debug, Default)]
pub struct RenderTexture {
    /// Render target texture binded to the internal [`rg::Pass`]
    tex: Texture2dDrop,
    /// Off-screen rendering pass
    pass: rg::Pass,
}

impl Drop for RenderTexture {
    fn drop(&mut self) {
        rg::Pass::destroy(self.pass);
    }
}

impl RenderTexture {
    pub fn builder(size: [u32; 2]) -> RenderTextureBuilder {
        RenderTextureBuilder::begin(size)
    }

    /// [`rokol::gfx::Pass`] for off-screen rendering
    pub fn pass(&self) -> rg::Pass {
        self.pass
    }

    pub fn tex(&self) -> &Texture2dDrop {
        &self.tex
    }

    pub fn img(&self) -> rg::Image {
        self.tex.img
    }
}
