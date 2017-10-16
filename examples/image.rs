extern crate image;
extern crate leechbar;

use leechbar::{Alignment, Background, BarBuilder, Component, Text, Width};
use std::time::Duration;

struct ImageComponent {
    index: u32,
    index_reset: u32,
    timeout: u64,
    alignment: Alignment,
}

impl Component for ImageComponent {
    fn background(&mut self) -> Option<Background> {
        let name = format!("./examples/testimages/image{}.png", self.index);
        let image = image::open(&name).unwrap();

        self.index += 2;
        if self.index > 9 {
            self.index = self.index_reset;
        }

        Some(Background::new_image(image))
    }

    fn alignment(&mut self) -> Alignment {
        self.alignment
    }

    fn text(&mut self) -> Option<Text> {
        let text = format!("Hello, World! {}", self.index);
        Some(Text::new(text))
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
        .background_color(255, 0, 255, 255)
        .foreground_color(0, 0, 0, 255)
        .background_image(image)
        .font("Fira Mono Medium 10")
        .height(30)
        .spawn()
        .unwrap();

    bar.add(ImageComponent {
        index: 0,
        index_reset: 0,
        timeout: 1110,
        alignment: Alignment::CENTER,
    });
    bar.add(ImageComponent {
        index: 1,
        index_reset: 1,
        timeout: 1000,
        alignment: Alignment::CENTER,
    });

    bar.start_event_loop();
}
