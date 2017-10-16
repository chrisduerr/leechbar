# Leechbar

**Warning**: This crate is WIP and might have some big changes in the future.

Leechbar is a crate for creating your own bar/panel/dock.

The goal of leechbar is to provide a library that allows creating a completely custom bar.
The purpose is not simplicity. So if you don't plan on using more than just simple text, you might
want to look at something like [lemonbar](https://github.com/LemonBoy/bar) instead.

### Usage

This crate can be installed through [crates.io](https://crates.io/crates/leechbar) and can be
used by adding it to your `Cargo.toml`.

```toml
[dependencies]
leechbar = "0.2.1"
```

### Examples

These snippets just touch the basics of using leechbar, for more complete examples you can take
a look at the [github repository](https://github.com/chrisduerr/leechbar/tree/master/examples).

The first thing that needs to be done for using leechbar, is setting up the bar configuration
itself. This is done using the `BarBuilder` struct.

```rust
use leechbar::BarBuilder;

// All method calls that take parameters are optional
BarBuilder::new()
    .background_color(255, 0, 255, 255)
    .font("Fira Mono Medium 14")
    .output("DVI-1")
    .height(30)
    .spawn();
```

After creating a configuration using `BarBuilder`, you have to add your components to the
bar. This is a little more complicated, because you need to implement the `Component` trait.

```rust
use leechbar::{BarBuilder, Component, Text, Background, Alignment, Width};
use std::time::Duration;

struct MyComponent;

// You can define your own custom components like this
impl Component for MyComponent {
    // No background image
    fn background(&mut self) -> Option<Background> {
        None
    }

    // Print "Hello, World!" as text
    fn text(&mut self) -> Option<Text> {
        Some(Text::new(String::from("Hello, World")))
    }

    // Put this element at the center of the bar
    fn alignment(&mut self) -> Alignment {
        Alignment::CENTER
    }

    // Do this only once
    fn timeout(&mut self) -> Option<Duration> {
        None
    }

    // No width restrictions
    fn width(&mut self) -> Width {
        Width::new()
    1}

    // Ignore all events
    fn event(&mut self) {}
}

// Create a new bar
let mut bar = BarBuilder::new().spawn().unwrap();
// Add an instance of your component to your bar
bar.add(MyComponent{});
// Start the event loop that handles all X events
bar.start_event_loop();
```
