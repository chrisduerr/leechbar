extern crate image;
extern crate leechbar;

use leechbar::component::{Alignment, Background, Component, ComponentPosition, Text};
use leechbar::BarBuilder;
use std::time::Duration;
use std::thread;

struct ImageComponent {
    index: u32,
    index_reset: u32,
    timeout: u64,
    position: ComponentPosition,
}

impl Component for ImageComponent {
    fn background(&mut self) -> Option<Background> {
        let name = format!("./testimages/image{}.png", self.index);
        let image = image::open(&name).unwrap();

        self.index += 2;
        if self.index > 9 {
            self.index = self.index_reset;
        }

        Some(Background::new_image(image))
    }

    fn position(&mut self) -> ComponentPosition {
        self.position
    }

    fn text(&mut self) -> Option<Text> {
        let text = format!("Hello, World! {}", self.index);
        Some(Text::new(text))
    }

    fn timeout(&mut self) -> Duration {
        Duration::from_millis(self.timeout)
    }

    fn event(&mut self) {}
}

fn main() {
    let image = image::open("img.png").unwrap();
    let mut bar = BarBuilder::new()
        .background_color(255, 0, 255, 255)
        .foreground_color(0, 0, 0, 255)
        .background_image(image)
        .font("Fira Mono Medium 10")
        .height(30)
        .spawn()
        .unwrap();

    bar.draw(ImageComponent {
        index: 0,
        index_reset: 0,
        timeout: 111,
        position: ComponentPosition::new(Alignment::CENTER, 0),
    });
    bar.draw(ImageComponent {
        index: 1,
        index_reset: 1,
        timeout: 100,
        position: ComponentPosition::new(Alignment::CENTER, 1),
    });

    loop {
        thread::sleep(Duration::from_secs(999_999));
    }
}
