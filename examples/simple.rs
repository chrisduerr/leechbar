extern crate env_logger;
extern crate leechbar;

use leechbar::{BarBuilder, Component, Foreground, Text};
use std::time::Duration;

struct MyComponent {
    text: Text,
}

impl Component for MyComponent {
    // Print "Hello, World!" as text
    fn foreground(&mut self) -> Option<Foreground> {
        Some(Foreground::new(&self.text))
    }
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
