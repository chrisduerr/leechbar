use image::{DynamicImage, GenericImage, Pixel};
use geometry::Geometry;
use picture::Picture;
use std::sync::Arc;
use error::*;
use bar::Bar;
use xcb;

/// A cached image.
///
/// This creates an image that is cached on the X server. Keeping this around instead of moving it
/// will usually lead to a lower CPU consumption but slightly increase the memory usage of the X
/// server.
#[derive(Clone)]
pub struct Image {
    pub(crate) arc: Arc<Picture>,
}

impl Image {
    /// Create a new image from a
    /// [`DynamicImage`](https://docs.rs/image/0.17.0/image/enum.DynamicImage.html).
    pub fn new(bar: &Bar, image: &DynamicImage) -> Result<Self> {
        let conn = Arc::clone(&bar.conn);
        let (window, gcontext, format32) = (bar.window, bar.gcontext, bar.format32);

        // Create shorthands for geometry
        let (w, h) = (image.width() as u16, image.height() as u16);

        // Create a pixmap for creating the picture
        let pix = conn.generate_id();
        xtry!(create_pixmap_checked, &conn, 32, pix, window, w, h);

        // Convert DynamicImage
        let data = convert_image(image);

        // Copy image data to pixmap
        xtry!(put_image_checked, &conn, 2u8, pix, gcontext, w, h, 0, 0, 0, 32, &data);

        // Create new picture from pixmap
        let picture = conn.generate_id();
        xtry!(@render create_picture_checked, &conn, picture, pix, format32, &[]);

        // Free the unneeded pixmap
        xcb::free_pixmap(&conn, pix);

        Ok(Self {
            arc: Arc::new(Picture {
                conn,
                xid: picture,
                geometry: Geometry::new(0, 0, w, h),
            }),
        })
    }
}

// Convert a DynamicImage to a raw image
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
