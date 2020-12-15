use {
    image::{io::Reader as ImageReader, GenericImageView},
    rokol::gfx::{self as rg, BakedResource},
    std::{io, path::Path},
};

fn gen_img(pixels: &[u8], size: [u32; 2]) -> rg::Image {
    rg::Image::create(&{
        let mut desc = rg::ImageDesc {
            type_: rg::ImageType::Dim2 as u32,
            width: size[0] as i32,
            height: size[1] as i32,
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
    pub fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let img = ImageReader::open(path)?
            .decode()
            // `io::Result` can upcast any other error type (just like `Box`)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // change u, v axes match to x, y axes
        #[cfg(rokol_gfx = "glcore33")]
        let img = img.flipv();

        let (w, h) = img.dimensions();
        let id = self::gen_img(img.as_bytes(), [w, h]);

        Ok(Self { img: id, w, h })
    }
}

impl crate::gfx::batcher::traits::Texture2d for TextureData2d {
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
