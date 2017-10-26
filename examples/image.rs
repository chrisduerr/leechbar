extern crate env_logger;
extern crate image;
extern crate leechbar;

use leechbar::{Alignment, Background, Bar, BarBuilder, Color, Component, Foreground, Image, Text,
               Timeout};
use std::time::Duration;
use std::ops::Range;

struct ImageComponent {
    bar: Bar,
    index: usize,
    timeout: u64,
    images: Vec<Image>,
    alignment: Alignment,
}

impl Component for ImageComponent {
    fn update(&mut self) -> bool {
        self.index += 1;
        if self.index >= self.images.len() {
            self.index = 0;
        }
        true
    }

    fn background(&self) -> Background {
        self.images[self.index].clone().into()
    }

    fn alignment(&self) -> Alignment {
        self.alignment
    }

    fn foreground(&self) -> Foreground {
        let content = format!("Hello, World! {}", self.index);
        Text::new(&self.bar, &content, None, None).unwrap().into()
    }

    fn timeout(&self) -> Option<Timeout> {
        Some(Timeout::new_duration(Duration::from_millis(self.timeout)))
    }
}

fn main() {
    // Start the logger
    env_logger::init().unwrap();

    let image = image::open("./examples/testimages/bg.png").unwrap();
    let mut bar = BarBuilder::new()
        .foreground_color(Color::new(0, 0, 0, 255))
        .background_image(image)
        .height(30)
        .spawn()
        .unwrap();

    let images = load_images(&bar, 0..4);
    let bar_clone = bar.clone();
    bar.add(ImageComponent {
        bar: bar_clone,
        images,
        index: 0,
        timeout: 3333,
        alignment: Alignment::CENTER,
    });
    let images = load_images(&bar, 5..9);
    let bar_clone = bar.clone();
    bar.add(ImageComponent {
        bar: bar_clone,
        images,
        index: 0,
        timeout: 3000,
        alignment: Alignment::CENTER,
    });

    bar.start_event_loop();
}

fn load_images(bar: &Bar, range: Range<u8>) -> Vec<Image> {
    let mut images = Vec::new();
    for i in range {
        let name = format!("./examples/testimages/image{}.png", i);
        let image = image::open(&name).unwrap();
        images.push(Image::new(bar, &image).unwrap());
    }
    images
}
