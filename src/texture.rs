use std::path::Path;
use image::{
    self,
    DynamicImage,
    GenericImage,
    RgbaImage,
};
use gfx::traits::*;
use { gfx, ImageSize, TextureSettings, TextureResult, TextureError,
    Rgba8Texture };

/// Represents a texture.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture<R> where R: gfx::Resources {
    handle: gfx::handle::Texture<R>
}

impl<R: gfx::Resources> Texture<R> {
    /// Gets a handle to the Gfx texture.
    pub fn handle(&self) -> gfx::handle::Texture<R> {
        self.handle.clone()
    }

    /// Returns empty texture.
    pub fn empty<F>(factory: &mut F) -> Result<Self, gfx::tex::TextureError>
        where F: gfx::Factory<R>
    {
        let tex_handle = try!(factory.create_texture_rgba8(1, 1));
        let ref image_info = tex_handle.get_info().clone().into();
        try!(factory.update_texture(
            &tex_handle,
            &image_info,
            &[0u8; 4],
            Some(gfx::tex::Kind::D2)
        ));
        Ok(Texture {
            handle: tex_handle
        })
    }

    /// Creates a texture from path.
    pub fn from_path<F, P>(
        factory: &mut F,
        path: P,
        settings: &TextureSettings,
    ) -> Result<Self, String>
        where F: gfx::Factory<R>,
              P: AsRef<Path>
    {
        let img = try!(image::open(path).map_err(|e| e.to_string()));

        let img = match img {
            DynamicImage::ImageRgba8(img) => img,
            img => img.to_rgba()
        };

        Ok(Texture::from_image(factory, &img, settings))
    }

    /// Creates a texture from image.
    pub fn from_image<F>(
        factory: &mut F,
        img: &RgbaImage,
        settings: &TextureSettings
    ) -> Self
        where F: gfx::Factory<R>
    {
        let (width, height) = img.dimensions();
        let tex_info = gfx::tex::TextureInfo {
            width: width as u16,
            height: height as u16,
            depth: 1,
            levels: 1,
            kind: gfx::tex::Kind::D2,
            format: if settings.get_convert_gamma() {
                        gfx::tex::Format::SRGB8_A8
                    } else { gfx::tex::RGBA8 }
        };
        let tex_handle = factory.create_texture_static(tex_info, &img).unwrap();
        if settings.get_generate_mipmap() {
            factory.generate_mipmap(&tex_handle);
        }
        Texture {
            handle: tex_handle
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
        let width = if width == 0 { 1 } else { width as u16 };
        let height = if height == 0 { 1 } else { height as u16 };

        let mut pixels = vec![];
        for alpha in buffer {
            pixels.extend(vec![255; 3]);
            pixels.push(*alpha);
        }

        let tex_handle = factory.create_texture_rgba8(width, height).unwrap();
        let ref image_info = tex_handle.get_info().clone().into();
        factory.update_texture(
            &tex_handle,
            &image_info,
            &pixels,
            Some(gfx::tex::Kind::D2)
        ).unwrap();

        Texture {
            handle: tex_handle
        }
    }

    /// Updates the texture with an image.
    pub fn update<F>(&mut self, factory: &mut F, image: &RgbaImage)
        where F: gfx::Factory<R>
    {
        factory.update_texture(&self.handle,
            &self.handle.get_info().clone().into(),
            &image,
            Some(gfx::tex::Kind::D2)
        ).unwrap();
    }
}

impl<F, R> Rgba8Texture<F> for Texture<R>
    where F: gfx::Factory<R>,
          R: gfx::Resources
{
    fn from_memory<S: Into<[u32; 2]>>(
        factory: &mut F,
        memory: &[u8],
        size: S,
        settings: &TextureSettings
    ) -> TextureResult<Self> {
        let size = size.into();
        let (width, height) = (size[0] as u16, size[1] as u16);
        let tex_info = gfx::tex::TextureInfo {
            width: width,
            height: height,
            depth: 1,
            levels: 1,
            kind: gfx::tex::Kind::D2,
            format: if settings.get_convert_gamma() {
                        gfx::tex::Format::SRGB8_A8
                    } else { gfx::tex::RGBA8 }
        };
        let tex_handle = match factory.create_texture_static(tex_info, &memory) {
            Ok(x) => x,
            Err(err) => {
                return Err(TextureError::FactoryError(format!("{:?}", err)));
            }
        };
        if settings.get_generate_mipmap() {
            factory.generate_mipmap(&tex_handle);
        }
        Ok(Texture { handle: tex_handle })
    }

    fn update<S: Into<[u32; 2]>>(
        &mut self,
        factory: &mut F,
        memory: &[u8],
        _size: S,
    ) -> TextureResult<()> {
        match factory.update_texture(&self.handle,
            &self.handle.get_info().clone().into(),
            &memory,
            Some(gfx::tex::Kind::D2)
        ) {
            Ok(()) => Ok(()),
            Err(err) => Err(TextureError::FactoryError(format!("{:?}", err)))
        }
    }
}

impl<R> ImageSize for Texture<R> where R: gfx::Resources {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let info = self.handle.get_info();
        (info.width as u32, info.height as u32)
    }
}
