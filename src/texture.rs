use gfx;
use image;
use image::{
    DynamicImage,
    GenericImage,
    ImageBuf,
    Rgba,
};
use texture_lib::ImageSize;

/// Represents a texture.
pub struct Texture {
    /// A handle to the Gfx texture.
    pub handle: gfx::TextureHandle,
}

impl Texture {
    /// Creates a texture from path.
    pub fn from_path<
        C: gfx::CommandBuffer,
        D: gfx::Device<C>
    >(device: &mut D, path: &Path) -> Result<Texture, String> {
        let img = match image::open(path) {
            Ok(img) => img,
            Err(e)  => return Err(format!("Could not load '{}': {}",
                path.filename_str().unwrap(), e)),
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
        let texture = device.create_texture(texture_info).unwrap();
        device.update_texture(&texture, &image_info,
            img.rawbuf().as_slice())
        .unwrap();

        Ok(Texture {
            handle: texture
        })
    }
    
    /// Creates a texture from image.
    pub fn from_image<
        C: gfx::CommandBuffer,
        D: gfx::Device<C>
    >(device: &mut D, image: &ImageBuf<Rgba<u8>>) -> Texture {
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
        let texture = device.create_texture(texture_info).unwrap();
        device.update_texture(&texture, &image_info,
            image.pixelbuf().as_slice())
        .unwrap();

        Texture {
            handle: texture
        }
    }

    /// Creates a texture from RGBA image.
    pub fn from_rgba8<D: gfx::Device<C>, C: gfx::CommandBuffer>(
        img: ImageBuf<Rgba<u8>>,
        d: &mut D
    ) -> Texture {
        let (width, height) = img.dimensions();

        let mut ti = gfx::tex::TextureInfo::new();
        ti.width = width as u16;
        ti.height = height as u16;
        ti.kind = gfx::tex::TextureKind::Texture2D;
        ti.format = gfx::tex::RGBA8;

        let tex = d.create_texture(ti).unwrap();
        d.update_texture(&tex, &ti.to_image_info(),
                         img.into_vec().as_slice()).unwrap();
        d.generate_mipmap(&tex);

        Texture {
            handle: tex,
        }
    }
   
    /// Creates texture from memory alpha. 
    pub fn from_memory_alpha<
        C: gfx::CommandBuffer,
        D: gfx::Device<C>
    >(
        device: &mut D,
        buffer: &[u8],
        width: u32,
        height: u32,
    ) -> Texture {
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
        let texture = device.create_texture(texture_info).unwrap();
        device.update_texture(&texture, &image_info,
            pixels.as_slice())
            .unwrap();
        Texture {
            handle: texture
        }
    }

    /// Updates the texture with an image.
    pub fn update<
        C: gfx::CommandBuffer,
        D: gfx::Device<C>
    >(&mut self, device: &mut D, image: &ImageBuf<Rgba<u8>>) {
        device.update_texture(&self.handle, 
            &self.handle.get_info().to_image_info(),
            image.pixelbuf().as_slice()
        ).unwrap();
    }

    /// Gets the size of the texture.
    #[inline(always)]
    pub fn get_size(&self) -> (u32, u32) {
        let info = self.handle.get_info();
        (info.width as u32, info.height as u32)
    }

    /// Gets the width of the texture.
    #[inline(always)]
    pub fn get_width(&self) -> u32 {
        let (w, _) = self.get_size();
        w
    }

    /// Gets the height of the texture.
    #[inline(always)]
    pub fn get_height(&self) -> u32 {
        let (_, h) = self.get_size();
        h
    }
}

impl ImageSize for Texture {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let info = self.handle.get_info();
        (info.width as u32, info.height as u32)
    }
}

