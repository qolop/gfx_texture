#![deny(missing_docs)]
#![feature(old_path)]

//! A Gfx texture representation that works nicely with Piston libraries.

extern crate gfx;
extern crate "texture" as texture_lib;
extern crate image;

pub use texture_lib::ImageSize;
pub use texture::Texture;

mod texture;

