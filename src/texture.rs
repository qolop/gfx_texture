use std::path::Path;
use { gfx, ImageSize, Settings };
use image::{
    self,
    DynamicImage,
    GenericImage,
    RgbaImage,
};

/// Represents a texture.
#[derive(Clone, Debug)]
pub struct Texture<R: gfx::Resources> {
    /// A handle to the Gfx texture.
    pub handle: gfx::TextureHandle<R>,
}

impl<R: gfx::Resources> Texture<R> {
    /// Creates a texture from path.
    pub fn from_path<F, P>(
        factory: &mut F,
        path: P,
        settings: &Settings,
    ) -> Result<Self, String>
        where F: gfx::Factory<R>,
              P: AsRef<Path>
    {
        let img = try!(image::open(path).map_err(|e| e.to_string()));

        //if settings.force_alpha //TODO
        let img = match img {
            DynamicImage::ImageRgba8(img) => img,
            img => img.to_rgba(),
        };

        let img = if settings.flip_vertical {
            image::imageops::flip_vertical(&img)
        } else {
            img
        };

        let texture = Texture::from_image(factory, &img,
                                          settings.compress,
                                          settings.generate_mipmap);
        Ok(texture)
    }

    /// Creates a texture from image.
    pub fn from_image<F: gfx::Factory<R>>(
        factory: &mut F,
        image: &RgbaImage,
        _compress: bool,
        generate_mipmap: bool
    ) -> Self {
        let (width, height) = image.dimensions();
        let texture_info = gfx::tex::TextureInfo {
            width: width as u16,
            height: height as u16,
            depth: 1,
            levels: 1,
            kind: gfx::tex::TextureKind::Texture2D,
            format: gfx::tex::RGBA8,
        };
        let image_info = texture_info.to_image_info();
        let texture = factory.create_texture(texture_info).unwrap();
        factory.update_texture(&texture, &image_info, &image,
                               Some(gfx::tex::TextureKind::Texture2D)).unwrap();
        if generate_mipmap {
            factory.generate_mipmap(&texture);
        }
        Texture {
            handle: texture
        }
    }

    /// Creates texture from memory alpha.
    pub fn from_memory_alpha<F: gfx::Factory<R>>(
        factory: &mut F,
        buffer: &[u8],
        width: u32,
        height: u32,
    ) -> Self {
        use std::cmp::max;

        let width = max(width, 1);
        let height = max(height, 1);

        let mut pixels = Vec::new();
        for alpha in buffer.iter() {
            pixels.push(255);
            pixels.push(255);
            pixels.push(255);
            pixels.push(*alpha);
        }

        let texture_info = gfx::tex::TextureInfo {
            width: width as u16,
            height: height as u16,
            depth: 1,
            levels: 1,
            kind: gfx::tex::TextureKind::Texture2D,
            format: gfx::tex::RGBA8,
        };

        let image_info = texture_info.to_image_info();
        let texture = factory.create_texture(texture_info).unwrap();
        factory.update_texture(&texture, &image_info, &pixels,
                               Some(gfx::tex::TextureKind::Texture2D)).unwrap();
        Texture {
            handle: texture
        }
    }

    /// Updates the texture with an image.
    pub fn update<F: gfx::Factory<R>>(&mut self, factory: &mut F, image: &RgbaImage) {
        factory.update_texture(&self.handle,
            &self.handle.get_info().to_image_info(),
            &image,
            Some(gfx::tex::TextureKind::Texture2D)
        ).unwrap();
    }
}

impl<R: gfx::Resources> ImageSize for Texture<R> {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let info = self.handle.get_info();
        (info.width as u32, info.height as u32)
    }
}
