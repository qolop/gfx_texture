use std::path::Path;

use gfx;
use image;
use image::{
    DynamicImage,
    GenericImage,
    RgbaImage,
};
use texture_lib::ImageSize;

/// Represents a texture.
#[derive(Clone, Debug)]
pub struct Texture<R: gfx::Resources> {
    /// A handle to the Gfx texture.
    pub handle: gfx::TextureHandle<R>,
}

impl<R: gfx::Resources> Texture<R> {
    /// Creates a texture from path.
    pub fn from_path<D: gfx::Factory<R>>(
        device: &mut D,
        path: &Path
    ) -> Result<Self, String> {
        let img = match image::open(path) {
            Ok(img) => img,
            Err(e)  => return Err(format!("Could not load '{:?}': {:?}",
                path.file_name().unwrap(), e)),
        };

        let img = match img {
            DynamicImage::ImageRgba8(img) => img,
            x => x.to_rgba()
        };

        let (width, height) = img.dimensions();
        let texture_info = gfx::tex::TextureInfo {
            width: width as u16,
            height: height as u16,
            depth: 1,
            levels: 1,
            kind: gfx::tex::TextureKind::Texture2D,
            format: gfx::tex::RGBA8,
        };
        let image_info = texture_info.to_image_info();
        let texture = device.create_texture(texture_info).ok().unwrap();
        device.update_texture(&texture, &image_info,
            img.as_ref())
        .ok().unwrap();

        Ok(Texture {
            handle: texture
        })
    }

    /// Creates a texture from image.
    pub fn from_image<D: gfx::Factory<R>>(
        device: &mut D,
        image: &RgbaImage
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
        let texture = device.create_texture(texture_info).ok().unwrap();
        device.update_texture(&texture, &image_info,
            image.as_ref())
        .ok().unwrap();

        Texture {
            handle: texture
        }
    }

    /// Creates a texture from image and generates mipmap.
    pub fn from_image_with_mipmap<D: gfx::Factory<R>>(
        device: &mut D,
        image: &RgbaImage
    ) -> Self {
        let texture = Texture::from_image(device, image);
        device.generate_mipmap(&texture.handle);

        texture
    }

    /// Creates texture from memory alpha.
    pub fn from_memory_alpha<D: gfx::Factory<R>>(
        device: &mut D,
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
        let texture = device.create_texture(texture_info).ok().unwrap();
        device.update_texture(&texture, &image_info,
            &pixels)
            .ok().unwrap();
        Texture {
            handle: texture
        }
    }

    /// Updates the texture with an image.
    pub fn update<D: gfx::Factory<R>>(&mut self, device: &mut D, image: &RgbaImage) {
        device.update_texture(&self.handle,
            &self.handle.get_info().to_image_info(),
            image.as_ref()
        ).ok().unwrap();
    }
}

impl<R: gfx::Resources> ImageSize for Texture<R> {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let info = self.handle.get_info();
        (info.width as u32, info.height as u32)
    }
}

