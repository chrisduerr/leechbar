//! TODO: Crate level documentation

#[macro_use]
extern crate error_chain;
extern crate image;
extern crate xcb;

pub mod error;
mod builder;
mod bar;

pub use builder::BarBuilder;
