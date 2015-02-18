#![deny(missing_docs)]
#![feature(path)]

//! A Gfx texture representation that works nicely with Piston libraries.

extern crate gfx;
extern crate "texture" as texture_lib;
extern crate image;

pub use texture::Texture;

mod texture;

