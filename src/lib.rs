#![deny(missing_docs)]

//! A Gfx texture representation that works nicely with Piston libraries.

extern crate gfx;
extern crate texture;
extern crate image;

pub use texture::*;

use std::path::Path;
use image::{
    DynamicImage,
    GenericImage,
    RgbaImage,
};
use gfx::traits::*;

/// Flip settings.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flip {
    /// Does not flip.
    None,
    /// Flips image vertically.
    Vertical,
}

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
        Rgba8Texture::create(factory, &[0u8; 4], [1, 1], &TextureSettings::new())
    }

    /// Creates a texture from path.
    pub fn from_path<F, P>(
        factory: &mut F,
        path: P,
        flip: Flip,
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

        let img = if flip == Flip::Vertical {
            image::imageops::flip_vertical(&img)
        } else {
            img
        };

        Texture::from_image(factory, &img, settings).map_err(
            |e| format!("{:?}", e))
    }

    /// Creates a texture from image.
    pub fn from_image<F>(
        factory: &mut F,
        img: &RgbaImage,
        settings: &TextureSettings
    ) -> Result<Self, gfx::tex::TextureError>
        where F: gfx::Factory<R>
    {
        let (width, height) = img.dimensions();
        Rgba8Texture::create(factory, img, [width, height], settings)
    }

    /// Creates texture from memory alpha.
    pub fn from_memory_alpha<F>(
        factory: &mut F,
        buffer: &[u8],
        width: u32,
        height: u32,
        settings: &TextureSettings
    ) -> Result<Self, gfx::tex::TextureError>
        where F: gfx::Factory<R>
    {
        if width == 0 || height == 0 {
            return Texture::empty(factory);
        }

        let size = [width, height];
        let buffer = texture::ops::alpha_to_rgba8(buffer, size);
        Rgba8Texture::create(factory, &buffer, size, settings)
    }

    /// Updates the texture with an image.
    pub fn update<F>(&mut self, factory: &mut F, img: &RgbaImage)
    -> Result<(), gfx::tex::TextureError>
        where F: gfx::Factory<R>
    {
        let (width, height) = img.dimensions();
        Rgba8Texture::update(self, factory, img, [width, height])
    }
}

impl<F, R> Rgba8Texture<F> for Texture<R>
    where F: gfx::Factory<R>,
          R: gfx::Resources
{
    type Error = gfx::tex::TextureError;

    fn create<S: Into<[u32; 2]>>(
        factory: &mut F,
        memory: &[u8],
        size: S,
        settings: &TextureSettings
    ) -> Result<Self, Self::Error> {
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
            Err(err) => { return Err(err); }
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
    ) -> Result<(), Self::Error> {
        factory.update_texture(&self.handle,
            &self.handle.get_info().clone().into(),
            &memory,
            Some(gfx::tex::Kind::D2)
        )
    }
}

impl<R> ImageSize for Texture<R> where R: gfx::Resources {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let info = self.handle.get_info();
        (info.width as u32, info.height as u32)
    }
}
