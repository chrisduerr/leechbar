use image::{DynamicImage, GenericImage, Pixel};
use builder::BarBuilder;
use xcb::{self, randr};
use std::sync::Arc;
use std::cmp;
use error::*;

// Utility macro for setting window properties
// This returns a Result
macro_rules! set_prop {
    ($conn:expr, $window:expr, $name:expr, @atom $value:expr) => {
        {
            match xcb::intern_atom($conn, true, $value).get_reply() {
                Ok(atom) => set_prop!($conn, $window, $name, &[atom.atom()], "ATOM", 32),
                Err(e) => Err(ErrorKind::XcbPropertyError(e.error_code())),
            }
        }
    };
    ($conn:expr, $window:expr, $name:expr, $data:expr) => {
        {
            set_prop!($conn, $window, $name, $data, "CARDINAL", 32)
        }
    };
    ($conn:expr, $window:expr, $name:expr, $data:expr, $type:expr, $size:expr) => {
        {
            let type_atom = xcb::intern_atom($conn, true, $type).get_reply();
            let property = xcb::intern_atom($conn, true, $name).get_reply();
            match (type_atom, property) {
                (Ok(type_atom), Ok(property)) => {
                    let property = property.atom();
                    let type_atom = type_atom.atom();
                    let mode = xcb::PROP_MODE_REPLACE as u8;
                    xcb::change_property($conn, mode, $window, property, type_atom, $size, $data);
                    Ok(())
                },
                (Err(e), _) | (_, Err(e)) => Err(ErrorKind::XcbPropertyError(e.error_code())),
            }
        }
    };
}

// Geometry of the bar
struct Geometry {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

impl Geometry {
    fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        Geometry {
            x,
            y,
            width,
            height,
        }
    }
}

pub struct Bar {
    conn: Arc<xcb::Connection>,
    geometry: Geometry,
    window: u32,
    gcontext: u32,
    background_pixmap: u32,
}

impl Bar {
    // Create a new bar
    pub fn new(builder: BarBuilder) -> Result<Self> {
        // Connect to the X server
        let conn = Arc::new(xcb::Connection::connect(None)?.0);

        // Create an empty skeleton bar
        let mut bar = Bar {
            conn,
            geometry: Geometry::new(0, 0, 0, 0),
            window: 0,
            gcontext: 0,
            background_pixmap: 0,
        };

        // Get geometry of the specified display
        let info = bar.screen_info(builder.output)?;
        bar.geometry = Geometry::new(info.x(), info.y(), info.width(), builder.height);

        // Create the window
        let name = builder.name.as_bytes();
        bar.create_window(builder.background_color, name)?;

        // Create background pixmap
        if let Some(background_image) = builder.background_image {
            bar.create_background_pixmap(background_image)?;
        }

        Ok(bar)
    }

    // Start the event loop
    // This is blocking
    pub fn start_event_loop(&self) {
        loop {
            self.conn.wait_for_event();
        }
    }

    fn screen(&self) -> Result<xcb::Screen> {
        self.conn
            .get_setup()
            .roots()
            .next()
            .ok_or_else(|| ErrorKind::XcbNoScreenError(()).into())
    }

    // Create a new window and set all required window parameters to make it a bar
    fn create_window(&mut self, background_color: u32, window_title: &[u8]) -> Result<()> {
        let conn = &self.conn;

        // Create the window
        let window = self.conn.generate_id();
        xcb::create_window(
            conn,
            xcb::WINDOW_CLASS_COPY_FROM_PARENT as u8,
            window,
            self.screen()?.root(),
            self.geometry.x,
            self.geometry.y,
            self.geometry.width,
            self.geometry.height,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            self.screen()?.root_visual(),
            &[
                (xcb::CW_BACK_PIXEL, background_color),
                (
                    xcb::CW_EVENT_MASK,
                    xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS |
                        xcb::EVENT_MASK_ENTER_WINDOW,
                ),
                (xcb::CW_OVERRIDE_REDIRECT, 0),
            ],
        );

        // Set all window properties
        set_prop!(conn, window, "_NET_WM_WINDOW_TYPE", @atom "_NET_WM_WINDOW_TYPE_DOCK")?;
        set_prop!(conn, window, "_NET_WM_STATE", @atom "_NET_WM_STATE_STICKY")?;
        set_prop!(conn, window, "_NET_WM_DESKTOP", &[-1])?;
        set_prop!(conn, window, "_NET_WM_NAME", window_title, "UTF8_STRING", 8)?;
        set_prop!(conn, window, "WM_NAME", window_title, "STRING", 8)?;

        // Request the WM to manage our window.
        xcb::map_window(conn, window);

        // Flush connection
        conn.flush();

        self.window = window;
        Ok(())
    }

    // Get information about specified output
    fn screen_info(
        &self,
        query_output_name: Option<String>,
    ) -> Result<xcb::Reply<xcb::ffi::randr::xcb_randr_get_crtc_info_reply_t>> {
        if query_output_name.is_none() {
            return self.primary_screen_info();
        }
        let query_output_name = query_output_name.unwrap(); // Safe unwrap

        // Load screen resources of the root window
        // Return result on error
        let res_cookie = randr::get_screen_resources(&self.conn, self.screen()?.root());
        let res_reply = res_cookie
            .get_reply()
            .map_err(|e| ErrorKind::XcbScreenResourcesError(e.error_code()))?;

        // Get all crtcs from the reply
        let crtcs = res_reply.crtcs();

        for crtc in crtcs {
            // Get info about crtc
            let crtc_info_cookie = randr::get_crtc_info(&self.conn, *crtc, 0);
            let crtc_info_reply = crtc_info_cookie.get_reply();

            if let Ok(reply) = crtc_info_reply {
                // Skip this crtc if it has no width or output
                if reply.width() == 0 || reply.num_outputs() == 0 {
                    continue;
                }

                // Get info of crtc's first output for output name
                let output = reply.outputs()[0];
                let output_info_cookie = randr::get_output_info(&self.conn, output, 0);
                let output_info_reply = output_info_cookie.get_reply();

                // Get the name of the first output
                let mut output_name = String::new();
                if let Ok(output_info_reply) = output_info_reply {
                    output_name = String::from_utf8_lossy(output_info_reply.name()).into();
                }

                // If the output name is the requested name, return the dimensions
                if output_name == query_output_name {
                    return Ok(reply);
                }
            }
        }

        let error_msg = ["Unable to find output '", &query_output_name, "'"].concat();
        Err(error_msg.into())
    }

    // Get information about the primary output
    fn primary_screen_info(
        &self,
    ) -> Result<xcb::Reply<xcb::ffi::randr::xcb_randr_get_crtc_info_reply_t>> {
        // Load primary output
        let output_cookie = randr::get_output_primary(&self.conn, self.screen()?.root());
        let output_reply = output_cookie
            .get_reply()
            .map_err(|e| ErrorKind::PrimaryScreenInfoError(e.error_code()))?;
        let output = output_reply.output();

        // Get crtc of primary output
        let output_info_cookie = randr::get_output_info(&self.conn, output, 0);
        let output_info_reply = output_info_cookie
            .get_reply()
            .map_err(|e| ErrorKind::PrimaryScreenInfoError(e.error_code()))?;
        let crtc = output_info_reply.crtc();

        // Get info of primary output's crtc
        let crtc_info_cookie = randr::get_crtc_info(&self.conn, crtc, 0);
        Ok(
            crtc_info_cookie
                .get_reply()
                .map_err(|e| ErrorKind::PrimaryScreenInfoError(e.error_code()))?,
        )
    }

    // Get the background pixmap with the image filled in
    fn create_background_pixmap(&mut self, background: DynamicImage) -> Result<()> {
        // Get depth of root
        let depth = self.screen()?.root_depth();

        // Make sure copied area is not bigger than image size
        let width = cmp::min(self.geometry.width, background.width() as u16);
        let height = cmp::min(self.geometry.height, background.height() as u16);

        // Create pixmap
        let pixmap = self.conn.generate_id();
        xcb::create_pixmap_checked(&self.conn, depth, pixmap, self.window, width, height)
            .request_check()
            .unwrap();

        // Create a graphics context
        let gcontext = self.conn.generate_id();
        xcb::create_gc_checked(&self.conn, gcontext, self.window, &[])
            .request_check()
            .unwrap();

        // Convert background image to raw pixel data
        let data = convert_image(background, u32::from(width), u32::from(height));

        // Put image data into pixmap
        xcb::put_image_checked(
            &self.conn,
            xcb::IMAGE_FORMAT_Z_PIXMAP as u8,
            pixmap,
            gcontext,
            width,
            height,
            0,
            0,
            0,
            depth,
            &data,
        ).request_check()
            .unwrap();

        // Copy background image to window
        xcb::copy_area_checked(
            &self.conn,
            pixmap,
            self.window,
            gcontext,
            0,
            0,
            0,
            0,
            width,
            height,
        ).request_check()
            .unwrap();

        self.background_pixmap = pixmap;
        self.gcontext = gcontext;

        Ok(())
    }
}

// Convert an image to a raw Vector that is cropped to a specific size
fn convert_image(mut image: DynamicImage, width: u32, height: u32) -> Vec<u8> {
    let mut image = image.crop(0, 0, width, height).to_rgba();

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
