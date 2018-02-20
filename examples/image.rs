// Image Example
//
// This example iterates over a few images which are stored in memory.

extern crate chan;
extern crate env_logger;
extern crate image;
extern crate leechbar;

use leechbar::{Background, Bar, BarBuilder, Color, Component, Foreground, Image, Text};
use std::time::Duration;
use std::ops::Range;
use std::thread;

// The component state required
struct ImageComponent {
    bar: Bar,
    index: usize,
    images: Vec<Image>,
}

// Create new component from bar and image cache
impl ImageComponent {
    fn new(bar: Bar, images: Vec<Image>) -> Self {
        Self {
            bar,
            images,
            index: 0,
        }
    }
}

// Implement the component trait
impl Component for ImageComponent {
    // Update the component state
    fn update(&mut self) -> bool {
        // Increase index and reset it when appropriate
        self.index += 1;
        if self.index >= self.images.len() {
            self.index = 0;
        }

        // Always redraw this component
        true
    }

    // Get the image from the cache and return it
    fn background(&self) -> Background {
        self.images[self.index].clone().into()
    }

    // Draw "Hello, World! <index>"
    // This text is not cached and re-rendered every time
    fn foreground(&self) -> Foreground {
        let content = format!("Hello, World! {}", self.index);

        // Create the text, this might fail but still require a redraw
        let text = Text::new(&self.bar, &content, None, None);
        if let Ok(text) = text {
            // If everything worked out, draw the text
            text.into()
        } else {
            // It's not possible to abort drawing here, so we draw no text as fallback
            // Updating in `update` would allow aborting this redraw and keep the last text
            Foreground::new()
        }
    }

    // Update component every three seconds
    fn redraw_timer(&mut self) -> chan::Receiver<()> {
        // Create a channel for sending out the redraw messages
        let (tx, rx) = chan::sync(0);

        // Spawn a new thread that loops forever
        thread::spawn(move || {
            loop {
                // Wait 3 seconds, then send an empty message to request a redraw
                thread::sleep(Duration::from_secs(3));
                tx.send(());
            }
        });

        rx
    }
}

fn main() {
    // Start the logger
    env_logger::init();

    // Select a background image for the bar
    let image = image::open("./examples/testimages/bg.png").expect("Unable to find bg image");
    let mut bar = BarBuilder::new()
        .foreground_color(Color::new(0, 0, 0, 255))
        .background_image(image)
        .spawn()
        .expect("Unable to spawn bar");

    // Load images into cache
    let images = load_images(&bar, 0..4);
    // Create component
    let comp = ImageComponent::new(bar.clone(), images);
    // Add component to bar
    bar.add(comp);

    // Load images into cache
    let images = load_images(&bar, 5..9);
    // Create component
    let comp = ImageComponent::new(bar.clone(), images);
    // Add component to bar
    bar.add(comp);

    // Start the event loop that handles all X events
    bar.start_event_loop();
}

// Helper for loading images into Xorg memory
fn load_images(bar: &Bar, range: Range<u8>) -> Vec<Image> {
    let mut images = Vec::new();
    for i in range {
        let name = format!("./examples/testimages/image{}.png", i);
        let image = image::open(&name).expect("Unable to find comp image");
        images.push(Image::new(bar, &image).expect("Unable to create X image"));
    }
    images
}
