use std::path::Path;
use { gfx, ImageSize, Settings };
use image::{
    self,
    DynamicImage,
    GenericImage,
    RgbaImage,
};

/// Represents a texture.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture<R> where R: gfx::Resources {
    handle: gfx::TextureHandle<R>
}

impl<R: gfx::Resources> Texture<R> {
    /// Get a handle to the Gfx texture.
    pub fn handle(&self) -> gfx::TextureHandle<R> {
        self.handle.clone()
    }

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
                                          settings.convert_gamma,
                                          settings.compress,
                                          settings.generate_mipmap);
        Ok(texture)
    }

    /// Creates a texture from image.
    pub fn from_image<F>(
        factory: &mut F,
        image: &RgbaImage,
        convert_gamma: bool,
        _compress: bool,
        generate_mipmap: bool
    ) -> Self
        where F: gfx::Factory<R>
    {
        let (width, height) = image.dimensions();
        let texture_info = gfx::tex::TextureInfo {
            width: width as u16,
            height: height as u16,
            depth: 1,
            levels: 1,
            kind: gfx::tex::TextureKind::Texture2D,
            format: if convert_gamma {
                gfx::tex::Format::SRGB8_A8
            } else { gfx::tex::RGBA8 }
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
    pub fn from_memory_alpha<F>(
        factory: &mut F,
        buffer: &[u8],
        width: u32,
        height: u32,
    ) -> Self
        where F: gfx::Factory<R>
    {
        let width = if width == 0 { 1 } else { width };
        let height = if height == 0 { 1 } else { height };

        let mut pixels = vec![];
        for alpha in buffer {
            pixels.extend(vec![255; 3]);
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
    pub fn update<F>(&mut self, factory: &mut F, image: &RgbaImage)
        where F: gfx::Factory<R>
    {
        factory.update_texture(&self.handle,
            &self.handle.get_info().to_image_info(),
            &image,
            Some(gfx::tex::TextureKind::Texture2D)
        ).unwrap();
    }
}

impl<R> ImageSize for Texture<R> where R: gfx::Resources {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let info = self.handle.get_info();
        (info.width as u32, info.height as u32)
    }
}
