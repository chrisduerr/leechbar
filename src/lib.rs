//! TODO: Crate level documentation

extern crate cairo;
extern crate cairo_sys;
#[macro_use]
extern crate error_chain;
extern crate image;
extern crate pango;
extern crate pangocairo;
extern crate xcb;

pub mod error;
pub mod component;
mod builder;
mod text;
mod bar;

pub use builder::BarBuilder;
