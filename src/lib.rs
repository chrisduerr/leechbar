//! TODO: Crate level documentation
#![recursion_limit = "1024"]

extern crate cairo;
extern crate cairo_sys;
#[macro_use]
extern crate error_chain;
extern crate image;
#[macro_use]
extern crate log;
extern crate pango;
extern crate pangocairo;
extern crate xcb;

#[macro_use]
mod macros;
pub mod error;
pub mod component;
mod bar_component;
mod geometry;
mod builder;
mod render;
mod util;
mod bar;

pub use builder::BarBuilder;
