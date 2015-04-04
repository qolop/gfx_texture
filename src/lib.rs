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
    /// Convert to rgba8.
    pub force_alpha: bool,
    /// Sometimes you need the other way around.
    pub flip_vertical: bool,
    /// Compress on GPU.
    pub compress: bool,
    /// Generate mipmap chain.
    pub generate_mipmap: bool,
}

impl Settings {
    /// Create default settings.
    pub fn new() -> Settings {
        Settings {
            force_alpha: false,
            flip_vertical: false,
            compress: false,
            generate_mipmap: false,
        }
    }
}
