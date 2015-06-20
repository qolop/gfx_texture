#![deny(missing_docs)]

//! A Gfx texture representation that works nicely with Piston libraries.

extern crate gfx;
extern crate texture as texture_lib;
extern crate image;

pub use texture_lib::ImageSize;
pub use texture::Texture;

mod texture;

/// Texture creation parameters.
pub struct Settings {
    flip_vertical: bool,
    convert_gamma: bool,
    /// Compress on GPU.
    compress: bool,
    /// Generate mipmap chain.
    generate_mipmap: bool,
}

impl Settings {
    /// Create default settings.
    pub fn new() -> Settings {
        Settings {
            flip_vertical: false,
            convert_gamma: false,
            compress: false,
            generate_mipmap: false,
        }
    }

    /// Gets whether to flip vertical.
    pub fn get_flip_vertical(&self) -> bool { self.flip_vertical }
    /// Sets flip vertical.
    pub fn set_flip_vertical(&mut self, val: bool) { self.flip_vertical = val; }
    /// Sets flip vertical.
    pub fn flip_vertical(mut self, val: bool) -> Self {
        self.set_flip_vertical(val);
        self
    }

    /// Gets wheter to convert gamma, treated as sRGB color space.
    pub fn get_convert_gamma(&self) -> bool { self.convert_gamma }
    /// Sets convert gamma.
    pub fn set_convert_gamma(&mut self, val: bool) { self.convert_gamma = val; }
    /// Sets convert gamma.
    pub fn convert_gamma(mut self, val: bool) -> Self {
        self.set_convert_gamma(val);
        self
    }

    /// Gets wheter compress on the GPU.
    pub fn get_compress(&self) -> bool { self.compress }
    /// Sets compress.
    pub fn set_compress(&mut self, val: bool) { self.compress = val; }
    /// Sets compress.
    pub fn compress(mut self, val: bool) -> Self {
        self.set_compress(val);
        self
    }

    /// Gets generate mipmap.
    pub fn get_generate_mipmap(&self) -> bool { self.generate_mipmap }
    /// Sets generate mipmap.
    pub fn set_generate_mipmap(&mut self, val: bool) {
        self.generate_mipmap = val;
    }
    /// Sets generate mipmap.
    pub fn generate_mipmap(mut self, val: bool) -> Self {
        self.set_generate_mipmap(val);
        self
    }
}
