#![deny(missing_docs)]

//! A Gfx texture representation that works nicely with Piston libraries.

extern crate gfx;
extern crate texture;
extern crate image;

pub use texture::*;

use std::path::Path;
use image::{
    DynamicImage,
    RgbaImage,
};
use gfx::CombinedError;
use gfx::format::{Srgba8, R8_G8_B8_A8};

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
        CreateTexture::create(factory, Format::Rgba8, &[0u8; 4], [1, 1],
                              &TextureSettings::new())
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
        CreateTexture::create(factory, Format::Rgba8,
                              img, [width, height], settings)
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
        CreateTexture::create(factory, Format::Rgba8, &buffer, size, settings)
    }

    /// Updates the texture with an image.
    pub fn update<C>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        img: &RgbaImage
    ) -> Result<(), gfx::UpdateError<[u16; 3]>>
        where C: gfx::CommandBuffer<R>
    {
        let (width, height) = img.dimensions();
        UpdateTexture::update(self, encoder, Format::Rgba8,
                              img, [width, height])
    }
}

impl<F, R> CreateTexture<F> for Texture<R>
    where F: gfx::Factory<R>,
          R: gfx::Resources
{
    type Error = CombinedError;

    fn create<S: Into<[u32; 2]>>(
        factory: &mut F,
        _format: Format,
        memory: &[u8],
        size: S,
        _settings: &TextureSettings
    ) -> Result<Self, Self::Error> {
        let size = size.into();
        let (width, height) = (size[0] as u16, size[1] as u16);
        let tex_kind = gfx::tex::Kind::D2(width, height,
            gfx::tex::AaMode::Single);

        let (surface, view) = try!(factory.create_texture_const_u8::<Srgba8>(
            tex_kind, &[memory]));
        Ok(Texture { surface: surface, view: view })
    }
}

impl<R, C> UpdateTexture<gfx::Encoder<R, C>> for Texture<R>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>
{
    type Error = gfx::UpdateError<[u16; 3]>;

    fn update<S: Into<[u32; 2]>>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        _format: Format,
        memory: &[u8],
        size: S,
    ) -> Result<(), Self::Error> {

        // TODO: This should be passed via the args.
        let offset = [0, 0];

        let size = size.into();
        let tex = &self.surface;
        let face = None;
        let img_info = gfx::tex::ImageInfoCommon {
            xoffset: offset[0],
            yoffset: offset[1],
            zoffset: 0,
            width: size[0] as u16,
            height: size[1] as u16,
            depth: 0,
            format: (),
            mipmap: 0,
        };
        let data = gfx::cast_slice(memory);

        encoder.update_texture::<_, Srgba8>(tex, face, img_info, data).map_err(Into::into)
    }
}

impl<R> ImageSize for Texture<R> where R: gfx::Resources {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let (w, h, _, _) = self.surface.get_info().kind.get_dimensions();
        (w as u32, h as u32)
    }
}
