use image::{DynamicImage, GenericImage, Pixel};
use std::sync::{Arc, Mutex};
use pango::FontDescription;
use component::Component;
use builder::BarBuilder;
use xcb::{self, randr};
use std::{cmp, thread};
use error::*;
use text;

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
#[derive(Clone, Copy)]
struct Geometry {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

impl Geometry {
    // Helper for creating a geometry without struct syntax
    fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        Geometry {
            x,
            y,
            width,
            height,
        }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

// A component currently stored in the bar
struct BarComponent {
    id: u32,
    picture: u32,
    geometry: Geometry,
}

impl BarComponent {
    // Creates a new component
    fn new(id: u32, conn: &Arc<xcb::Connection>) -> Self {
        let picture = conn.generate_id();
        BarComponent {
            geometry: Geometry::default(),
            picture,
            id,
        }
    }

    // Update a component cached by the bar
    fn set_geometry(&mut self, geometry: Geometry) {
        self.geometry = geometry;
    }

    // Redraw a component
    // Copies the pixmap to the window
    fn redraw(&self, bar: &Bar) -> Result<()> {
        let (w, h, x) = (self.geometry.width, self.geometry.height, self.geometry.x);
        bar.composite_picture(self.picture, 0, x, w, h)?;
        Ok(())
    }

    // Clear the area of this component
    // This should be called before updating it
    fn clear(&self, bar: &Bar) -> Result<()> {
        let (w, h, x) = (self.geometry.width, self.geometry.height, self.geometry.x);
        if bar.background != 0 {
            // Copy image if background exists
            bar.composite_picture(bar.background, x, x, w, h)?;
        } else {
            // Clear rectangle if there is no background image
            xcb::clear_area_checked(&bar.conn, false, bar.window, x, 0, w, h)
                .request_check()
                .map_err(|e| format!("Unable to clear component: {}", e))?;
        }

        Ok(())
    }
}

// The main bar struct for keeping state
#[derive(Clone)]
pub struct Bar {
    conn: Arc<xcb::Connection>,
    geometry: Geometry,
    window: u32,
    window_pict: u32,
    gcontext: u32,
    background: u32,
    font: Option<String>,
    components: Arc<Mutex<Vec<BarComponent>>>,
    format32: u32,
    format24: u32,
    color: (f64, f64, f64, f64),
}

impl Bar {
    // Create a new bar
    pub fn new(builder: BarBuilder) -> Result<Self> {
        // Connect to the X server
        let conn = Arc::new(xcb::Connection::connect(None)?.0);

        // Global text foreground color
        let color = builder.foreground_color;

        // Get geometry of the specified display
        let info = screen_info(&conn, builder.output)?;
        let geometry = Geometry::new(info.x(), info.y(), info.width(), builder.height);

        // Create the window
        let name = builder.name.as_bytes();
        let window = create_window(&conn, geometry, builder.background_color, name)?;

        // Get 24 bit and 32 bit image formats
        let (format24, format32) = image_formats(&conn)?;

        // Create a GC with 32 bit depth
        let gcontext = {
            // First create a dummy pixmap with 32 bit depth
            let pix32 = conn.generate_id();
            xcb::create_pixmap_checked(&conn, 32, pix32, window, 1, 1)
                .request_check()
                .map_err(|e| format!("Unable to create GC dummy pixmap: {}", e))?;

            // Then create a gc from that pixmap
            let gc = conn.generate_id();
            xcb::create_gc_checked(&conn, gc, pix32, &[])
                .request_check()
                .map_err(|e| format!("Unable to create GC: {}", e))?;
            gc
        };

        // Create picture for the window
        let window_pict = conn.generate_id();
        xcb::render::create_picture_checked(&conn, window_pict, window, format24, &[])
            .request_check()
            .map_err(|e| format!("Unable to create window picture: {}", e))?;

        // Create background picture
        let background = if let Some(background_image) = builder.background_image {
            // Get width and height for the picture
            let w = background_image.width() as u16;
            let h = background_image.height() as u16;

            // Create a pixmap for creating the picture
            let pix = conn.generate_id();
            xcb::create_pixmap_checked(&conn, 32, pix, window, w, h)
                .request_check()
                .unwrap();

            // Canvert the image to the right format
            let data = convert_image(&background_image);

            // Copy image data to pixmap
            xcb::put_image_checked(&conn, 2u8, pix, gcontext, w, h, 0, 0, 0, 32, &data)
                .request_check()
                .unwrap();

            // Create new picture from pixmap
            let bg = conn.generate_id();
            xcb::render::create_picture_checked(&conn, bg, pix, format32, &[])
                .request_check()
                .unwrap();

            // Free the unneeded pixmap
            xcb::free_pixmap(&conn, pix);

            bg
        } else {
            0
        };

        // Create an empty skeleton bar
        Ok(Bar {
            conn,
            color,
            window,
            geometry,
            gcontext,
            format24,
            format32,
            background,
            window_pict,
            font: builder.font,
            components: Arc::new(Mutex::new(Vec::new())),
        })
    }

    // Start the event loop
    pub fn start_event_loop(&self) {
        let bar = self.clone();
        thread::spawn(move || {
            loop {
                if let Some(event) = bar.conn.wait_for_event() {
                    let r = event.response_type();
                    if r == xcb::EXPOSE {
                        // Composite bg over bar again if the image exists
                        if bar.background != 0 {
                            let (w, h) = (bar.geometry.width, bar.geometry.height);
                            bar.composite_picture(bar.background, 0, 0, w, h).unwrap();
                        };

                        // Redraw components
                        let components = bar.components.lock().unwrap();
                        for component in &*components {
                            let geometry = component.geometry;
                            if geometry.width > 0 && geometry.height > 0 {
                                component.redraw(&bar).unwrap();
                            }
                        }
                    } else {
                        // TODO: Handle mouse events
                    }
                }
            }
        });
    }

    // Handle drawing and updating a single element
    // Starts a new thread
    pub fn draw<T: 'static + Component + Send>(&mut self, mut component: T) {
        // Permanent component id
        let id = component.position().unique_id();

        // Register the component
        let bar_component = BarComponent::new(id, &self.conn);
        {
            let mut components = self.components.lock().unwrap();
            (*components).push(bar_component);
        }

        // Start bar thread
        let bar = self.clone();
        thread::spawn(move || {
            // Font has to be created for every thread because `FontDescription` is not `Send`
            let font = if let Some(ref font) = bar.font {
                FontDescription::from_string(font)
            } else {
                FontDescription::new()
            };

            // Shorten a few properties for the massive xcb methods
            let (conn, gc, win) = (&bar.conn, bar.gcontext, bar.window);

            // Start component loop
            loop {
                // Get new text and background from component
                let background = component.background();
                let mut text = component.text();

                // Shadow the font to make temporary override possible
                let mut font = font.clone();

                // Calculate width and height of element
                let h = bar.geometry.height;
                let mut w = 0;
                if let Some(ref background) = background {
                    if let Some(ref image) = background.image {
                        w = image.width() as u16;
                    }
                }
                if let Some(ref mut text) = text {
                    // Set fallback font and color
                    if let Some(ref font_override) = text.font {
                        font = FontDescription::from_string(font_override);
                    }
                    if text.color.is_none() {
                        text.color = Some(bar.color);
                    }

                    let text_width = text::text_size(&text.content, &font).unwrap().0;
                    w = cmp::max(w, text_width);
                }
                w = cmp::min(w, bar.geometry.width);

                // Prevents component from being redrawn while pixmap is freed
                // Lock components
                let mut components = bar.components.lock().unwrap();

                // Create pixmap and fill it with transparent pixels
                let pix = conn.generate_id();
                xcb::create_pixmap_checked(conn, 32, pix, win, w, h)
                    .request_check()
                    .unwrap();
                xcb::poly_fill_rectangle_checked(conn, pix, gc, &[xcb::Rectangle::new(0, 0, w, h)])
                    .request_check()
                    .unwrap();

                // Add background to pixmap
                if let Some(background) = background {
                    // Copy color if there is a color
                    if let Some(color) = background.color {
                        // Create a GC with the color
                        let col_gc = conn.generate_id();
                        xcb::create_gc_checked(
                            conn,
                            col_gc,
                            pix,
                            &[(xcb::ffi::xproto::XCB_GC_FOREGROUND, color)],
                        ).request_check()
                            .unwrap();

                        // Fill the pixmap with the GC color
                        xcb::poly_fill_rectangle_checked(
                            conn,
                            pix,
                            col_gc,
                            &[xcb::Rectangle::new(0, 0, w, h)],
                        ).request_check()
                            .unwrap();
                    }

                    // Copy image if there is an image
                    if let Some(image) = background.image {
                        // Convert image to raw pixels
                        let data = convert_image(&image);

                        // Get width and height of the image
                        let iw = image.width() as u16;
                        let ih = image.height() as u16;

                        // Get X position
                        let x = background.alignment.x_offset(w, iw);

                        // Put image on pixmap
                        xcb::put_image_checked(conn, 2u8, pix, gc, iw, ih, x, 0, 0, 32, &data)
                            .request_check()
                            .unwrap();
                    }
                }

                // Add text to pixmap
                if let Some(text) = text {
                    let screen = screen(conn).unwrap();
                    text::render_text(conn, &screen, pix, w, h, &font, &text);
                }

                // TODO: If width did not change, just clear and redraw this single component

                // Get the X offset of the first item that will be redrawn
                let mut x = xoffset_by_id(&(*components), id, w, bar.geometry.width);

                // Get all components that need to be redrawn
                components.sort_by(|a, b| a.id.cmp(&b.id));
                let components = components
                    .iter_mut()
                    .filter(|c| (c.id % 3 != 0 || c.id >= id) && c.id % 3 == id % 3)
                    .collect::<Vec<&mut BarComponent>>();

                // Remove all selected components from the bar
                for component in &components {
                    component.clear(&bar).unwrap();
                }

                // Redraw all selected components
                for component in components {
                    // Old rectangle for clearing bar
                    let (w, h) = if component.id == id {
                        // Update picture with the new pixmap
                        let pict = component.picture;
                        xcb::render::free_picture(conn, pict);
                        xcb::render::create_picture_checked(conn, pict, pix, bar.format32, &[])
                            .request_check()
                            .unwrap();
                        (w, h)
                    } else {
                        (component.geometry.width, component.geometry.height)
                    };

                    // Update the component
                    component.set_geometry(Geometry::new(x, 0, w, h));

                    // Redraw the component
                    if w > 0 && h > 0 {
                        component.redraw(&bar).unwrap();
                        x += w as i16;
                    }
                }

                // Flush XCB Connection
                conn.flush();

                // Sleep
                thread::sleep(component.timeout());
            }
        });
    }

    // Composite a picture on top of the background
    fn composite_picture(&self, pic: u32, srcx: i16, tarx: i16, w: u16, h: u16) -> Result<()> {
        // Shorten window to make xcb call single-line
        let win = self.window_pict;

        // Composite pictures
        let op = xcb::render::PICT_OP_OVER as u8;
        xcb::render::composite_checked(&self.conn, op, pic, 0, win, srcx, 0, 0, 0, tarx, 0, w, h)
            .request_check()
            .map_err(|e| format!("Unable to composite pictures: {}", e))?;

        // Flush connection
        self.conn.flush();

        Ok(())
    }
}

// Convert an image to a raw Vector that is cropped to a specific size
fn convert_image(image: &DynamicImage) -> Vec<u8> {
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

// Component's X-Offset by id
// If id is from center component, will return new X of the first component
fn xoffset_by_id(components: &[BarComponent], id: u32, new_width: u16, bar_width: u16) -> i16 {
    // Check if component is not left-aligned
    if id % 3 != 0 {
        // Filter unrelevant components
        let components = components
            .iter()
            .filter(|c| c.id != id && c.id % 3 == id % 3);

        // Get new width of all components
        let mut width = f64::from(components.map(|c| c.geometry.width).sum::<u16>());
        width += f64::from(new_width);

        if id % 3 == 1 {
            // Center
            (f64::from(bar_width) / 2f64 - width / 2f64) as i16
        } else {
            // Right
            bar_width as i16 - width as i16
        }
    } else {
        // Return selected component's old X
        components
            .iter()
            .filter(|c| id > c.id && c.id % 3 == id % 3)
            .map(|c| c.geometry.width)
            .sum::<u16>() as i16
    }
}

// Get the 24 and 32 bit image formats
// Response is Result<(format24, format32)>
fn image_formats(conn: &Arc<xcb::Connection>) -> Result<(u32, u32)> {
    // Query connection for all available formats
    let formats = xcb::render::query_pict_formats(conn)
        .get_reply()
        .map_err(|e| format!("Unable to query picture formats: {}", e))?
        .formats();

    let mut format24 = None;
    let mut format32 = None;
    for fmt in formats {
        let direct = fmt.direct();

        // Update 32 bit format if the format matches
        if fmt.depth() == 32 && direct.alpha_shift() == 24 && direct.red_shift() == 16
            && direct.green_shift() == 8 && direct.blue_shift() == 0
        {
            format32 = Some(fmt);
        }

        // Update 24 bit format if the format matches
        if fmt.depth() == 24 && direct.red_shift() == 16 && direct.green_shift() == 8
            && direct.blue_shift() == 0
        {
            format24 = Some(fmt);
        }

        // Stop iteration when matches have been found
        if format32.is_some() && format24.is_some() {
            break;
        }
    }

    // Error if one of the formats hasn't been found
    match (format24, format32) {
        (Some(f_24), Some(f_32)) => Ok((f_24.id(), f_32.id())),
        _ => Err("Unable to find picture formats".into()),
    }
}

// Get information about specified output
fn screen_info(
    conn: &Arc<xcb::Connection>,
    query_output_name: Option<String>,
) -> Result<xcb::Reply<xcb::ffi::randr::xcb_randr_get_crtc_info_reply_t>> {
    let root = screen(conn)?.root();

    // Return the default screen when no output is specified
    if query_output_name.is_none() {
        return primary_screen_info(conn, root);
    }
    let query_output_name = query_output_name.unwrap(); // Safe unwrap

    // Load screen resources of the root window
    // Return result on error
    let res_cookie = randr::get_screen_resources(conn, root);
    let res_reply = res_cookie
        .get_reply()
        .map_err(|e| ErrorKind::XcbScreenResourcesError(e.error_code()))?;

    // Get all crtcs from the reply
    let crtcs = res_reply.crtcs();

    for crtc in crtcs {
        // Get info about crtc
        let crtc_info_cookie = randr::get_crtc_info(conn, *crtc, 0);
        let crtc_info_reply = crtc_info_cookie.get_reply();

        if let Ok(reply) = crtc_info_reply {
            // Skip this crtc if it has no width or output
            if reply.width() == 0 || reply.num_outputs() == 0 {
                continue;
            }

            // Get info of crtc's first output for output name
            let output = reply.outputs()[0];
            let output_info_cookie = randr::get_output_info(conn, output, 0);
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
    conn: &Arc<xcb::Connection>,
    root: u32,
) -> Result<xcb::Reply<xcb::ffi::randr::xcb_randr_get_crtc_info_reply_t>> {
    // Load primary output
    let output_cookie = randr::get_output_primary(conn, root);
    let output_reply = output_cookie
        .get_reply()
        .map_err(|e| ErrorKind::PrimaryScreenInfoError(e.error_code()))?;
    let output = output_reply.output();

    // Get crtc of primary output
    let output_info_cookie = randr::get_output_info(conn, output, 0);
    let output_info_reply = output_info_cookie
        .get_reply()
        .map_err(|e| ErrorKind::PrimaryScreenInfoError(e.error_code()))?;
    let crtc = output_info_reply.crtc();

    // Get info of primary output's crtc
    let crtc_info_cookie = randr::get_crtc_info(conn, crtc, 0);
    Ok(
        crtc_info_cookie
            .get_reply()
            .map_err(|e| ErrorKind::PrimaryScreenInfoError(e.error_code()))?,
    )
}

// Create a new window and set all required window parameters to make it a bar
fn create_window(
    conn: &Arc<xcb::Connection>,
    geometry: Geometry,
    background_color: u32,
    window_title: &[u8],
) -> Result<u32> {
    // Get screen of connection
    let screen = screen(conn)?;

    // Create the window
    let window = conn.generate_id();
    xcb::create_window(
        conn,
        xcb::WINDOW_CLASS_COPY_FROM_PARENT as u8,
        window,
        screen.root(),
        geometry.x,
        geometry.y,
        geometry.width,
        geometry.height,
        0,
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &[
            (xcb::CW_BACK_PIXEL, background_color),
            (
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS | xcb::EVENT_MASK_ENTER_WINDOW,
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

    Ok(window)
}

// Used to get the screen of the connection
// TODO: Cache this instead of getting it from conn every time
fn screen(conn: &Arc<xcb::Connection>) -> Result<xcb::Screen> {
    conn.get_setup()
        .roots()
        .next()
        .ok_or_else(|| ErrorKind::XcbNoScreenError(()).into())
}
