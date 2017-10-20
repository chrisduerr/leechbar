extern crate image;
extern crate leechbar;

use leechbar::{Alignment, Background, Bar, BarBuilder, Color, Component, Foreground, Image, Text,
               Width};
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
    fn background(&mut self) -> Background {
        let bg = Background::new().image(self.images[self.index].clone());

        self.index += 1;
        if self.index >= self.images.len() {
            self.index = 0;
        }

        bg
    }

    fn alignment(&mut self) -> Alignment {
        self.alignment
    }

    fn foreground(&mut self) -> Option<Foreground> {
        let content = format!("Hello, World! {}", self.index);
        let text = Text::new(&self.bar, &content, None, None).unwrap();
        Some(Foreground::new(&text))
    }

    fn timeout(&mut self) -> Option<Duration> {
        Some(Duration::from_millis(self.timeout))
    }

    fn width(&mut self) -> Width {
        Width::new()
    }

    fn event(&mut self) {}
}

fn main() {
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
