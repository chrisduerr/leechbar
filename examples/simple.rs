// Simple Example
//
// This example draws "Hello, World!" and then doesn't redraw anymore.

extern crate env_logger;
extern crate leechbar;

use leechbar::{BarBuilder, Component, Foreground, Text};

struct MyComponent {
    text: Text,
}

impl Component for MyComponent {
    // Print "Hello, World!" as text
    fn foreground(&self) -> Foreground {
        self.text.clone().into()
    }
}

fn main() {
    // Start the logger
    env_logger::init().unwrap();

    // Create a new bar
    let mut bar = BarBuilder::new().spawn().unwrap();

    // Create a text for the component
    // Like this it is stored in memory and the `MyComponent` struct does not need `bar`
    let text = Text::new(&bar, "Hello, World!", None, None).unwrap();

    // Add an instance of your component to your bar
    bar.add(MyComponent { text });

    // Start the event loop that handles all X events
    bar.start_event_loop();
}
