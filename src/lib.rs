//! Leechbar is a crate for creating your own bar/panel/dock.
//!
//! The goal of leechbar is to provide a library that allows creating a completely custom bar.
//! The purpose is not simplicity. So if you don't plan on using more than just simple text, you might
//! want to look at something like [lemonbar](https://github.com/LemonBoy/bar) instead.
//!
//! # Usage
//!
//! This crate can be installed through [crates.io](https://crates.io/crates/leechbar) and can be
//! used by adding it to your `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! leechbar = "0.1.0"
//! ```
//!
//! # Examples
//!
//! These snippets just touch the basics of using leechbar, for more complete examples you can take
//! a look at the [github repository](https://github.com/chrisduerr/leechbar/tree/master/examples).
//!
//! The first thing that needs to be done for using leechbar, is setting up the bar configuration
//! itself. This is done using the [`BarBuilder`] struct.
//!
//! ```rust,no_run
//! use leechbar::BarBuilder;
//!
//! // All method calls that take parameters are optional
//! BarBuilder::new()
//!     .background_color(255, 0, 255, 255)
//!     .font("Fira Mono Medium 14")
//!     .output("DVI-1")
//!     .height(30)
//!     .spawn()
//!     .unwrap();
//! ```
//!
//! After creating a configuration using [`BarBuilder`], you have to add your components to the
//! bar. This is a little more complicated, because you need to implement the [`Component`] trait.
//!
//! ```rust,no_run
//! use leechbar::{BarBuilder, Component, Text, Background, ComponentPosition, Alignment, Width};
//! use std::time::Duration;
//!
//! struct MyComponent;
//!
//! // You can define your own custom components like this
//! impl Component for MyComponent {
//!     // No background image
//!     fn background(&mut self) -> Option<Background> {
//!         None
//!     }
//!
//!     // Print "Hello, World!" as text
//!     fn text(&mut self) -> Option<Text> {
//!         Some(Text::new(String::from("Hello, World")))
//!     }
//!
//!     // First element on the left side
//!     fn position(&mut self) -> ComponentPosition {
//!         ComponentPosition::new(Alignment::CENTER, 0)
//!     }
//!
//!     // Do this only once
//!     fn timeout(&mut self) -> Option<Duration> {
//!         None
//!     }
//!
//!     // No width restrictions
//!     fn width(&mut self) -> Width {
//!         Width::new()
//!     }
//!
//!     // Ignore all events
//!     fn event(&mut self) {}
//! }
//!
//! // Create a new bar
//! let mut bar = BarBuilder::new().spawn().unwrap();
//! // Add an instance of your component to your bar
//! bar.add(MyComponent);
//! // Start the event loop that handles all X events
//! bar.start_event_loop();
//! ```
//!
//! [`BarBuilder`]: struct.BarBuilder.html
//! [`Component`]: component/trait.Component.html
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
mod bar_component;
mod component;
mod geometry;
mod builder;
mod render;
mod util;
mod bar;

pub use builder::BarBuilder;
pub use component::*;
pub use bar::Bar;
