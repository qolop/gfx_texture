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
        use gfx::traits::*;

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
        settings: &Settings,
    ) -> Result<Self, String>
        where F: gfx::Factory<R>,
              P: AsRef<Path>
    {
        let img = try!(image::open(path).map_err(|e| e.to_string()));

        //if settings.force_alpha //TODO
        let mut img = match img {
            DynamicImage::ImageRgba8(img) => img,
            img => img.to_rgba()
        };

        if settings.flip_vertical {
            img = image::imageops::flip_vertical(&img);
        }

        Ok(Texture::from_image(factory, &img,
                               settings.convert_gamma,
                               settings.compress,
                               settings.generate_mipmap))
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
        let tex_info = gfx::tex::TextureInfo {
            width: width as u16,
            height: height as u16,
            depth: 1,
            levels: 1,
            kind: gfx::tex::Kind::D2,
            format: if convert_gamma {
                gfx::tex::Format::SRGB8_A8
            } else { gfx::tex::RGBA8 }
        };
        let tex_handle = factory.create_texture_static(tex_info, &image).unwrap();
        if generate_mipmap {
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
        use gfx::traits::*;

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

impl<R> ImageSize for Texture<R> where R: gfx::Resources {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let info = self.handle.get_info();
        (info.width as u32, info.height as u32)
    }
}
