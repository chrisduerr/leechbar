extern crate image;
extern crate leechbar;

use leechbar::BarBuilder;

fn main() {
    let image = image::open("image.png").unwrap();
    BarBuilder::new()
        .background_color(255, 0, 255, 255)
        .foreground_color(0, 255, 0, 255)
        .background_image(image)
        .height(30)
        .spawn()
        .unwrap();
}
