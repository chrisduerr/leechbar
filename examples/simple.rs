extern crate env_logger;
extern crate leechbar;

use leechbar::{Alignment, Background, BarBuilder, Component, Foreground, Text, Width};
use std::time::Duration;

struct MyComponent {
    text: Text,
}

impl Component for MyComponent {
    // No background image
    fn background(&mut self) -> Background {
        Background::new()
    }

    // Print "Hello, World!" as text
    fn foreground(&mut self) -> Option<Foreground> {
        Some(Foreground::new(&self.text))
    }

    // First element on the left side
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
    }

    // Ignore all events
    fn event(&mut self) {}
}

fn main() {
    // Start the logger
    env_logger::init().unwrap();

    // Create a new bar
    let mut bar = BarBuilder::new().spawn().unwrap();

    // Create a text for the component
    // This stores the rendered text in memory and prevents excessive redrawing
    let text = Text::new(&bar, "Hello, World!", None, None).unwrap();

    // Add an instance of your component to your bar
    bar.add(MyComponent { text });

    // Start the event loop that handles all X events
    bar.start_event_loop();
}
