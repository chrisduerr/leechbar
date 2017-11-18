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
//! leechbar = "0.2.1"
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
//! use leechbar::{BarBuilder, Color};
//!
//! // All method calls that take parameters are optional
//! BarBuilder::new()
//!     .background_color(Color::new(255, 0, 255, 255))
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
//! use leechbar::{Bar, BarBuilder, Component, Text, Foreground};
//!
//! struct MyComponent {
//!     bar: Bar,
//! }
//!
//! // You can define your own custom components like this
//! impl Component for MyComponent {
//!     // Print "Hello, World!" as text
//!     fn foreground(&self) -> Foreground {
//!         Text::new(&self.bar, "Hello, World", None, None).unwrap().into()
//!     }
//! }
//!
//! // Create a new bar
//! let mut bar = BarBuilder::new().spawn().unwrap();
//!
//! // Create an instance of the component
//! let comp = MyComponent { bar: bar.clone() };
//!
//! // Add an instance of your component to your bar
//! bar.add(comp);
//!
//! // Start the event loop that handles all X events
//! bar.start_event_loop();
//! ```
//!
//! # Logging
//!
//! This crate supports [`log`], if you want to enable this logging, you can add [`env_logger`] to
//! your binary.
//!
//! ```rust
//! extern crate env_logger;
//!
//! fn main() {
//!     env_logger::init().unwrap();
//!     // All the cool bar stuff
//! }
//! ```
//!
//! [`log`]: https://docs.rs/log
//! [`env_logger`]: http://rust-lang-nursery.github.io/log/env_logger
//! [`BarBuilder`]: struct.BarBuilder.html
//! [`Component`]: component/trait.Component.html
#![recursion_limit = "1024"]

extern crate cairo;
extern crate cairo_sys;
#[macro_use]
extern crate chan;
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
mod component;
mod error;
mod builder;
mod render;
mod event;
mod util;
mod bar;

pub use event::{ClickEvent, Event, MotionEvent, MouseButton};
pub use component::foreground::Foreground;
pub use component::background::Background;
pub use component::alignment::Alignment;
pub use error::{BarError, BarErrorKind};
pub use component::width::Width;
pub use component::text::Text;
pub use component::img::Image;
pub use component::Component;
pub use builder::BarBuilder;
pub use util::color::Color;
pub use bar::Bar;
