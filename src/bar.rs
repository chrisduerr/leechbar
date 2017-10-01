use image::{DynamicImage, GenericImage, Pixel};
use std::sync::{Arc, Mutex};
use component::Component;
use builder::BarBuilder;
use xcb::{self, randr};
use std::thread;
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

// A component currently stored in the bar
struct BarComponent {
    pixmap: u32,
    width: u16,
    height: u16,
    x: i16,
    id: u32,
}

impl BarComponent {
    // Creates a new component
    fn new(id: u32, conn: &Arc<xcb::Connection>) -> Self {
        let pixmap = conn.generate_id();
        BarComponent {
            pixmap,
            width: 0,
            height: 0,
            x: 0,
            id,
        }
    }

    // Update a component cached by the bar
    fn update(&mut self, x: i16, width: u16, height: u16) {
        self.height = height;
        self.width = width;
        self.x = x;
    }

    // Redraw a component
    // Copies the pixmap to the window
    fn redraw(&self, conn: &Arc<xcb::Connection>, window: u32, gc: u32, x: i16) -> Result<()> {
        let w = self.width;
        let h = self.height;
        xcb::copy_area_checked(conn, self.pixmap, window, gc, 0, 0, x, 0, w, h)
            .request_check()
            .map_err(|e| format!("Unable to redraw component: {}", e))?;

        Ok(())
    }

    // Clear the area of this component
    // This should be called before updating it
    fn clear(&self, conn: &Arc<xcb::Connection>, window: u32, gc: u32, bg: u32) -> Result<()> {
        let (w, h, x) = (self.width, self.height, self.x);
        if bg != 0 {
            // Copy image if background exists
            xcb::copy_area_checked(conn, bg, window, gc, x, 0, x, 0, w, h)
                .request_check()
                .map_err(|e| format!("Unable to clear component: {}", e))?;
        } else {
            // Clear rectangle if there is no background image
            xcb::clear_area_checked(conn, false, window, x, 0, w, h)
                .request_check()
                .map_err(|e| format!("Unable to clear component: {}", e))?;
        }

        Ok(())
    }
}

// The main bar struct for keeping state
pub struct Bar {
    conn: Arc<xcb::Connection>,
    geometry: Geometry,
    depth: u8,
    window: u32,
    gcontext: u32,
    background_pixmap: u32,
    components: Arc<Mutex<Vec<BarComponent>>>,
}

// TODO: Add clone to bar so the let (.,.,.) = (self.,self.,self.) stuff can be simplified
// TODO: This should clone conn and components and copy everything else
impl Bar {
    // Create a new bar
    pub fn new(builder: BarBuilder) -> Result<Self> {
        // Connect to the X server
        let conn = Arc::new(xcb::Connection::connect(None)?.0);

        // Create an empty skeleton bar
        let mut bar = Bar {
            conn,
            geometry: Geometry::new(0, 0, 0, 0),
            depth: 0,
            window: 0,
            gcontext: 0,
            background_pixmap: 0,
            components: Arc::new(Mutex::new(Vec::new())),
        };

        // Get geometry of the specified display
        let info = bar.screen_info(builder.output)?;
        bar.geometry = Geometry::new(info.x(), info.y(), info.width(), builder.height);

        // Create the window
        let name = builder.name.as_bytes();
        bar.create_window(builder.background_color, name)?;

        // Create a graphics context
        bar.gcontext = bar.conn.generate_id();
        xcb::create_gc_checked(&bar.conn, bar.gcontext, bar.window, &[])
            .request_check()
            .unwrap();

        // Get depth of root
        // Don't move this, it's required for creating the bg pixmap
        bar.depth = bar.screen()?.root_depth();

        // Create background pixmap
        if let Some(background_image) = builder.background_image {
            bar.create_background_pixmap(background_image)?;
        }

        Ok(bar)
    }

    // Start the event loop
    pub fn start_event_loop(&self) {
        let conn = Arc::clone(&self.conn);
        let components = Arc::clone(&self.components);
        let (window, gc, bg, geometry) = (
            self.window,
            self.gcontext,
            self.background_pixmap,
            self.geometry,
        );
        thread::spawn(move || {
            loop {
                // self.conn.wait_for_event();
                if let Some(event) = conn.wait_for_event() {
                    let r = event.response_type();
                    if r == xcb::EXPOSE {
                        // Redraw background if it's an image
                        if bg != 0 {
                            let (w, h) = (geometry.width, geometry.height);
                            xcb::copy_area_checked(&conn, bg, window, gc, 0, 0, 0, 0, w, h)
                                .request_check()
                                .unwrap();
                        };

                        // Redraw components
                        let components = components.lock().unwrap();
                        for component in &*components {
                            if component.width > 0 && component.height > 0 {
                                component.redraw(&conn, window, gc, component.x).unwrap();
                            }
                        }
                    } else {
                        // TODO
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
        let pixmap = bar_component.pixmap;
        {
            let mut components = self.components.lock().unwrap();
            (*components).push(bar_component);
        }

        // Start bar thread
        let conn = Arc::clone(&self.conn);
        let components = Arc::clone(&self.components);
        let (depth, window, gc, background, bar_width) = (
            self.depth,
            self.window,
            self.gcontext,
            self.background_pixmap,
            self.geometry.width,
        );
        thread::spawn(move || {
            loop {
                // Get background
                let image: Option<DynamicImage> = component.background();
                if let Some(image) = image {
                    // Calculate width and height
                    let w = image.width() as u16;
                    let h = image.height() as u16;

                    // Convert image to raw pixels
                    let data = convert_image(&image);

                    // Prevents component from being redrawn while pixmap is freed
                    // Lock components
                    let mut components = components.lock().unwrap();

                    // Free the old pixmap
                    xcb::free_pixmap(&conn, pixmap);

                    // Update pixmap
                    xcb::create_pixmap_checked(&conn, depth, pixmap, window, w, h)
                        .request_check()
                        .map_err(|e| format!("Unable to create component pixmap: {}", e))
                        .unwrap();

                    // Put image
                    xcb::put_image_checked(&conn, 2u8, pixmap, gc, w, h, 0, 0, 0, depth, &data)
                        .request_check()
                        .unwrap();

                    // Get the X offset of the first item that will be redrawn
                    let mut x = xoffset_by_id(&(*components), id, w, bar_width);

                    // Get all components that need to be redrawn
                    components.sort_by(|a, b| a.id.cmp(&b.id));
                    let components = components
                        .iter_mut()
                        .filter(|c| (c.id % 3 != 0 || c.id >= id) && c.id % 3 == id % 3)
                        .collect::<Vec<&mut BarComponent>>();

                    // Remove all selected components from the bar
                    for component in &components {
                        component.clear(&conn, window, gc, background).unwrap();
                    }

                    // Redraw all selected components
                    for component in components {
                        // Old rectangle for clearing bar
                        let (w, h) = if component.id == id {
                            (w, h)
                        } else {
                            (component.width, component.height)
                        };

                        // Update the component
                        component.update(x, w, h);

                        // Redraw the component
                        if w > 0 && h > 0 {
                            component.redraw(&conn, window, gc, x).unwrap();
                            x += w as i16;
                        }
                    }
                }

                // Sleep
                thread::sleep(component.timeout());
            }
        });
    }

    // Used to get the screen of the connection
    // TODO: Cache this instead of getting it from conn every time
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
                    xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS
                        | xcb::EVENT_MASK_ENTER_WINDOW,
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
        // Bar's gcontext and depth
        let depth = self.depth;
        let gc = self.gcontext;

        // Make sure copied area is not bigger than image size
        let h = background.height() as u16;
        let w = background.width() as u16;

        // Create pixmap
        let pixmap = self.conn.generate_id();
        xcb::create_pixmap_checked(&self.conn, self.depth, pixmap, self.window, w, h)
            .request_check()
            .unwrap();

        // Convert background image to raw pixel data
        let data = convert_image(&background);

        // Put image data into pixmap
        xcb::put_image_checked(&self.conn, 2u8, pixmap, gc, w, h, 0, 0, 0, depth, &data)
            .request_check()
            .unwrap();

        // Copy background image to window
        xcb::copy_area_checked(&self.conn, pixmap, self.window, gc, 0, 0, 0, 0, w, h)
            .request_check()
            .unwrap();

        self.background_pixmap = pixmap;

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
        let mut width = components.map(|c| c.width).sum::<u16>() as f64;
        width += new_width as f64;

        if id % 3 == 1 {
            // Center
            (bar_width as f64 / 2f64 - width / 2f64) as i16
        } else {
            // Right
            bar_width as i16 - width as i16
        }
    } else {
        // Return selected component's old X
        components
            .iter()
            .filter(|c| id > c.id && c.id % 3 == id % 3)
            .map(|c| c.width)
            .sum::<u16>() as i16
    }
}
