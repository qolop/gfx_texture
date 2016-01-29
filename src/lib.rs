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
use gfx::core::factory::CombinedError;
use gfx::format::{Rgba8, R8_G8_B8_A8};

/// Flip settings.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flip {
    /// Does not flip.
    None,
    /// Flips image vertically.
    Vertical,
}

/// Represents a texture.
pub struct Texture<R> where R: gfx::Resources {
    /// Pixel storage for texture.
    pub surface: gfx::handle::Texture<R, R8_G8_B8_A8>,
    /// View used by shader.
    pub view: gfx::handle::ShaderResourceView<R, [f32; 4]>
}

impl<R: gfx::Resources> Texture<R> {
    /// Returns empty texture.
    pub fn empty<F>(factory: &mut F) -> Result<Self, CombinedError>
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
    ) -> Result<Self, CombinedError>
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
    ) -> Result<Self, CombinedError>
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
    -> Result<(), CombinedError>
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
    type Error = CombinedError;

    fn create<S: Into<[u32; 2]>>(
        factory: &mut F,
        memory: &[u8],
        size: S,
        settings: &TextureSettings
    ) -> Result<Self, Self::Error> {
        let size = size.into();
        let (width, height) = (size[0] as u16, size[1] as u16);
        let tex_info = gfx::tex::Kind::D2(width, height,
            gfx::tex::AaMode::Single);

        let (surface, view) = try!(factory.create_texture_const::<Rgba8>(
            tex_info, gfx::cast_slice(memory),
            settings.get_generate_mipmap()));
        Ok(Texture { surface: surface, view: view })
    }

    fn update<S: Into<[u32; 2]>>(
        &mut self,
        factory: &mut F,
        memory: &[u8],
        _size: S,
    ) -> Result<(), Self::Error> {
        factory.update_texture::<Rgba8>(&self.surface,
            &self.surface.get_info().to_image_info(0),
            gfx::cast_slice(memory),
            None
        ).map_err(|err| err.into())
    }
}

impl<R> ImageSize for Texture<R> where R: gfx::Resources {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let (w, h, _, _) = self.surface.get_info().kind.get_dimensions();
        (w as u32, h as u32)
    }
}
