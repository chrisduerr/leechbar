use image::{DynamicImage, Pixel};
use std::sync::Arc;
use error::*;
use xcb;

// Convert an image to a raw Vector that is cropped to a specific size
pub fn convert_image(image: &DynamicImage) -> Vec<u8> {
    let mut image = image.to_rgba();

    // Correct channels to fit xorg layout
    for pixel in image.pixels_mut() {
        let channels = pixel.channels_mut();
        let tmp0 = channels[2];
        let tmp2 = channels[0];
        channels[0] = tmp0;
        channels[2] = tmp2;
    }

    image.into_raw()
}

// Get the u32 color from four u8 colors
pub fn color(red: u8, green: u8, blue: u8, alpha: u8) -> u32 {
    ((u32::from(alpha)) << 24) + ((u32::from(red)) << 16) + ((u32::from(green)) << 8)
        + u32::from(blue)
}

// Get the screen from an XCB Connection
pub fn screen(conn: &Arc<xcb::Connection>) -> Result<xcb::Screen> {
    conn.get_setup()
        .roots()
        .next()
        .ok_or_else(|| ErrorKind::XcbNoScreenError(()).into())
}
