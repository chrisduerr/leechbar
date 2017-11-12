// Time Example
//
// This example is a simple HH:MM clock.

extern crate chan;
extern crate env_logger;
extern crate leechbar;
extern crate time;

use leechbar::{Bar, BarBuilder, Component, Foreground, Text};
use std::time::Duration;
use std::thread;

// Store important state for the component
pub struct Time {
    bar: Bar,
    last_content: String,
    last_text: Option<Text>,
}

// Create the component from a `Bar`
// This can be obtained by calling `.clone()` on your bar
impl Time {
    pub fn new(bar: Bar) -> Self {
        Self {
            bar,
            last_text: None,
            last_content: String::new(),
        }
    }
}

// Implement all necessary trait methods
impl Component for Time {
    // In here the new time is calculated
    // If the time changed, the component will redraw
    fn update(&mut self) -> bool {
        // Get the current time and format it
        let time = time::now();
        let content = format!("{:02}:{:02}", time.tm_hour, time.tm_min);

        // Check if the time has changed since the last draw
        if content != self.last_content {
            // Create a new text using X
            let text = Text::new(&self.bar, &content, None, None);
            if let Ok(text) = text {
                // If the creation was successful, add the new text
                // Otherwise don't change the text
                self.last_text = Some(text);

                // Update the `last_content` for checking the next time
                self.last_content = content;
            }

            // Redraw after the content has changed
            true
        } else {
            // Don't redraw if nothing has changed
            false
        }
    }

    // Draw the text that's currently stored in `self`
    fn foreground(&self) -> Foreground {
        // Check if text is `None`
        if let Some(ref last_text) = self.last_text {
            // Draw text if it exists
            last_text.clone().into()
        } else {
            // Draw empty foreground if text is `None`
            Foreground::new()
        }
    }

    // Update component every 5 seconds
    // It will only redraw when the time has changed and `update` returns `true`
    fn redraw_timer(&mut self) -> chan::Receiver<()> {
        // Create a channel for sending out the redraw messages
        let (tx, rx) = chan::sync(0);

        // Spawn a new thread that loops forever
        thread::spawn(move || loop {
            // Wait 5 seconds, then send an empty message to request a redraw
            thread::sleep(Duration::from_secs(5));
            tx.send(());
        });

        rx
    }
}

fn main() {
    // Start the logger
    env_logger::init().expect("Unable to start logger");

    // Create a new bar
    let mut bar = BarBuilder::new().spawn().expect("Unable to spawn bar");

    // Add an instance of the component to the bar
    let comp = Time::new(bar.clone());
    bar.add(comp);

    // Start the event loop that handles all X events
    bar.start_event_loop();
}
