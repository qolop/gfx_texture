#![deny(missing_docs)]

//! A Gfx texture representation that works nicely with Piston libraries.

extern crate gfx;
extern crate gfx_core;
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
    /// Sampler for texture.
    pub sampler: gfx::handle::Sampler<R>,
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
        let offset = [0, 0];
        let size = [width, height];
        UpdateTexture::update(self, encoder, Format::Rgba8, img, offset, size)
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
        settings: &TextureSettings
    ) -> Result<Self, Self::Error> {
        // Modified `Factory::create_texture_immutable_u8` for dynamic texture.
        fn create_texture<T, F, R>(
            factory: &mut F,
            kind: gfx::texture::Kind,
            data: &[&[u8]]
        ) -> Result<(
            gfx::handle::Texture<R, T::Surface>,
            gfx::handle::ShaderResourceView<R, T::View>
        ), CombinedError>
            where F: gfx::Factory<R>,
                  R: gfx::Resources,
                  T: gfx::format::TextureFormat
        {
            use gfx::{format, texture};
            use gfx::memory::{Usage, SHADER_RESOURCE};
            use gfx_core::memory::Typed;

            let surface = <T::Surface as format::SurfaceTyped>::get_surface_type();
            let num_slices = kind.get_num_slices().unwrap_or(1) as usize;
            let num_faces = if kind.is_cube() {6} else {1};
            let desc = texture::Info {
                kind: kind,
                levels: (data.len() / (num_slices * num_faces)) as texture::Level,
                format: surface,
                bind: SHADER_RESOURCE,
                usage: Usage::Dynamic,
            };
            let cty = <T::Channel as format::ChannelTyped>::get_channel_type();
            let raw = try!(factory.create_texture_raw(desc, Some(cty), Some(data)));
            let levels = (0, raw.get_info().levels - 1);
            let tex = Typed::new(raw);
            let view = try!(factory.view_texture_as_shader_resource::<T>(
                &tex, levels, format::Swizzle::new()
            ));
            Ok((tex, view))
        }

        let size = size.into();
        let (width, height) = (size[0] as u16, size[1] as u16);
        let tex_kind = gfx::texture::Kind::D2(width, height,
            gfx::texture::AaMode::Single);

        // FIXME Use get_min too. gfx has only one filter setting for both.
        let filter_method = match settings.get_mag() {
            texture::Filter::Nearest => gfx::texture::FilterMethod::Scale,
            texture::Filter::Linear => gfx::texture::FilterMethod::Bilinear,
        };
        let sampler_info = gfx::texture::SamplerInfo::new(
            filter_method,
            gfx::texture::WrapMode::Clamp
        );

        let (surface, view) = try!(create_texture::<Srgba8, F, R>(
            factory, tex_kind, &[memory])
        );
        let sampler = factory.create_sampler(sampler_info);
        Ok(Texture { surface: surface, sampler: sampler, view: view })
    }
}

impl<R, C> UpdateTexture<gfx::Encoder<R, C>> for Texture<R>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>
{
    type Error = gfx::UpdateError<[u16; 3]>;

    fn update<O, S>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        format: Format,
        memory: &[u8],
        offset: O,
        size: S,
    ) -> Result<(), Self::Error>
        where O: Into<[u32; 2]>,
              S: Into<[u32; 2]>,
    {
        let offset = offset.into();
        let size = size.into();
        let tex = &self.surface;
        let face = None;
        let img_info = gfx::texture::ImageInfoCommon {
            xoffset: offset[0] as u16,
            yoffset: offset[1] as u16,
            zoffset: 0,
            width: size[0] as u16,
            height: size[1] as u16,
            depth: 0,
            format: (),
            mipmap: 0,
        };
        let data = gfx::memory::cast_slice(memory);

        match format {
            Format::Rgba8 => {
                use gfx::format::Rgba8;
                encoder.update_texture::<_, Rgba8>(tex, face, img_info, data).map_err(Into::into)
            },
        }
    }
}

impl<R> ImageSize for Texture<R> where R: gfx::Resources {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let (w, h, _, _) = self.surface.get_info().kind.get_dimensions();
        (w as u32, h as u32)
    }
}
