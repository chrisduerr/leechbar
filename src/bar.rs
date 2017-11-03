use image::{DynamicImage, GenericImage};
use xcb::{self, randr, Rectangle};
use bar_component::BarComponent;
use std::sync::{Arc, Mutex};
use component::Component;
use builder::BarBuilder;
use geometry::Geometry;
use event::Event;
use color::Color;
use std::thread;
use error::*;
use render;
use chan;
use util;
use img;

/// The main bar.
#[derive(Clone)]
pub struct Bar {
    pub(crate) conn: Arc<xcb::Connection>,
    pub(crate) geometry: Geometry,
    pub(crate) window: u32,
    pub(crate) window_pict: u32,
    pub(crate) gcontext: u32,
    pub(crate) background: u32,
    pub(crate) font: Option<String>,
    pub(crate) components: Arc<Mutex<Vec<BarComponent>>>,
    pub(crate) format32: u32,
    pub(crate) format24: u32,
    pub(crate) color: Color,
    pub(crate) component_ids: [u32; 3],
    pub(crate) text_yoffset: i16,
}

impl Bar {
    // Create a new bar
    pub(crate) fn new(builder: BarBuilder) -> Result<Self> {
        // Connect to the X server
        let conn = Arc::new(xcb::Connection::connect(None)?.0);

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
                .map_err(|e| ErrorKind::XError(format!("Unable to create GC dummy pixmap: {}", e)))?;

            // Then create a gc from that pixmap
            let gc = conn.generate_id();
            xcb::create_gc_checked(&conn, gc, pix32, &[])
                .request_check()
                .map_err(|e| ErrorKind::XError(format!("Unable to create GC: {}", e)))?;

            // Free pixmap after creating the gc
            xcb::free_pixmap_checked(&conn, pix32)
                .request_check()
                .map_err(|e| ErrorKind::XError(format!("Unable to free GC dummy pixmap: {}", e)))?;

            gc
        };

        // Create picture for the window
        let window_pict = conn.generate_id();
        xcb::render::create_picture_checked(&conn, window_pict, window, format24, &[])
            .request_check()
            .map_err(|e| ErrorKind::XError(format!("Unable to create window picture: {}", e)))?;

        // Create background picture
        let (bg_col, bg_img) = (builder.background_color, builder.background_image);
        let background =
            create_background_picture(&conn, window, gcontext, format32, geometry, bg_col, bg_img)?;

        // Create an empty skeleton bar
        Ok(Bar {
            conn,
            window,
            geometry,
            gcontext,
            format24,
            format32,
            background,
            window_pict,
            font: builder.font,
            component_ids: [0, 1, 2],
            color: builder.foreground_color,
            text_yoffset: builder.text_yoffset,
            components: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Start the event loop of the bar. This handles all X.Org events and is blocking.
    ///
    /// It **must** be called after adding all your components.
    pub fn start_event_loop(&self) {
        info!("Started event loop");
        loop {
            if let Some(event) = self.conn.wait_for_event() {
                let r = event.response_type();
                if r == xcb::EXPOSE {
                    debug!("Received expose event, redrawing…");

                    // Composite bg over self again if the image exists
                    let (w, h) = (self.geometry.width, self.geometry.height);
                    let res = self.composite_picture(self.background, 0, 0, w, h);
                    err!(res, "Unable to composite background");

                    // Redraw components
                    let components = self.components.lock().unwrap();
                    for component in &*components {
                        let geometry = component.geometry;
                        if geometry.width > 0 && geometry.height > 0 {
                            let res = component.redraw(self);
                            err!(res, "Unable to redraw component");
                        }
                    }
                } else if r == xcb::MOTION_NOTIFY {
                    let event: &xcb::MotionNotifyEvent = unsafe { xcb::cast_event(&event) };
                    debug!("Mouse moved to {}-{}", event.event_x(), event.event_y());
                    self.propagate_event(event.into());
                } else if r == xcb::BUTTON_PRESS {
                    let event: &xcb::ButtonPressEvent = unsafe { xcb::cast_event(&event) };
                    debug!("Mouse button {} pressed at {}", event.detail(), event.event_x());
                    self.propagate_event(event.into());
                }
            }
        }
    }

    pub fn propagate_event(&self, event: Event) {
        let x = match event {
            Event::ClickEvent(ref e) => e.position.x,
            Event::MotionEvent(ref e) => e.position.x,
        };

        let components = self.components.lock().unwrap();
        for component in &(*components) {
            let geo = component.geometry;
            if geo.x < x && geo.x as u16 + geo.width > x as u16 {
                if let Some(ref interrupt) = component.interrupt {
                    debug!("Event propagated to component {}", component.id);
                    interrupt.send(event);
                }
                break;
            }
        }
    }

    /// Add a new component to the bar.
    #[allow(unused_mut)]
    pub fn add<T: 'static + Component + Send>(&mut self, mut component: T) {
        // Permanent component id
        let id = component.alignment().id(&mut self.component_ids);

        debug!("Adding component {}", id);

        // Register the component
        let bar_component = BarComponent::new(id, &self.conn);
        {
            let mut components = self.components.lock().unwrap();
            (*components).push(bar_component);
        }

        // Start bar thread
        let bar = self.clone();
        thread::spawn(move || {
            // Get the polling receiver from the component
            let redraw_timer = component.redraw_timer();

            // Start component loop
            loop {
                // Check if component should be redrawn
                if component.update() {
                    let res = render::render(&bar, &mut component, id);
                    err!(res, "Component {}", id);
                }

                // Update the interrupt on the component
                let (tx, rx) = chan::async();
                {
                    let mut components = bar.components.lock().unwrap();
                    let comp_index = components.binary_search_by_key(&id, |c| c.id).unwrap_or(0);
                    components[comp_index].interrupt = Some(tx.clone());
                }

                // Select between redraw and event receivers
                // Redraw when requested
                loop {
                    chan_select! {
                        rx.recv() -> event => {
                            if let Some(event) = event {
                                debug!("Component {} received event.", id);
                                if component.event(event) {
                                    debug!("Component {} requested redraw after event.", id);
                                    break;
                                }
                            }
                        },
                        redraw_timer.recv() -> ping => {
                            if ping.is_some() {
                                debug!("Component {} requested redraw without event.", id);
                                break;
                            } else {
                                debug!("Component {} disconnected.", id);
                                return;
                            }
                        },
                    }
                }
            }
        });
    }

    // Composite a picture on top of the background
    pub(crate) fn composite_picture(
        &self,
        pic: u32,
        srcx: i16,
        tarx: i16,
        w: u16,
        h: u16,
    ) -> Result<()> {
        // Shorten window to make xcb call single-line
        let win = self.window_pict;

        // Composite pictures
        let op = xcb::render::PICT_OP_OVER as u8;
        xcb::render::composite_checked(&self.conn, op, pic, 0, win, srcx, 0, 0, 0, tarx, 0, w, h)
            .request_check()
            .map_err(|e| ErrorKind::XError(format!("Unable to composite picture: {}", e)))?;

        Ok(())
    }
}

// Get the 24 and 32 bit image formats
// Response is Result<(format24, format32)>
fn image_formats(conn: &Arc<xcb::Connection>) -> Result<(u32, u32)> {
    // Query connection for all available formats
    let formats = xcb::render::query_pict_formats(conn)
        .get_reply()
        .map_err(|e| ErrorKind::XError(format!("Unable to query picture formats: {}", e)))?
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
    let root = util::screen(conn)?.root();

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
    Ok(crtc_info_cookie
        .get_reply()
        .map_err(|e| ErrorKind::PrimaryScreenInfoError(e.error_code()))?)
}

// Create a new window and set all required window parameters to make it a bar
fn create_window(
    conn: &Arc<xcb::Connection>,
    geometry: Geometry,
    background_color: Color,
    window_title: &[u8],
) -> Result<u32> {
    // Get screen of connection
    let screen = util::screen(conn)?;

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
            (xcb::CW_BACK_PIXEL, background_color.into()),
            (
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_POINTER_MOTION
                    | xcb::EVENT_MASK_BUTTON_PRESS,
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

    info!("Created bar window");

    Ok(window)
}

// Create the picture that contains the background color/image
fn create_background_picture(
    conn: &Arc<xcb::Connection>,
    window: u32,
    gcontext: u32,
    format32: u32,
    geometry: Geometry,
    bg_color: Color,
    background_image: Option<DynamicImage>,
) -> Result<u32> {
    // Create shorthands for geometry
    let (w, h) = (geometry.width, geometry.height);

    // Create a pixmap for creating the picture
    let pix = conn.generate_id();
    xcb::create_pixmap_checked(conn, 32, pix, window, w, h)
        .request_check()
        .map_err(|e| ErrorKind::XError(format!("Unable to create pixmap for bg image: {}", e)))?;

    // Add the color to the pixmap
    // Create a GC with the color
    let col_gc = conn.generate_id();
    xtry!(
        create_gc_checked,
        conn,
        col_gc,
        pix,
        &[(xcb::ffi::xproto::XCB_GC_FOREGROUND, bg_color.into())]
    );

    // Fill the pixmap with the GC color
    xtry!(poly_fill_rectangle_checked, conn, pix, col_gc, &[Rectangle::new(0, 0, w, h)]);

    // Free gc after filling the rectangle
    xcb::free_gc(conn, col_gc);

    // Add image to pixmap
    if let Some(background_image) = background_image {
        // Get width and height for the picture
        let w = background_image.width() as u16;
        let h = background_image.height() as u16;

        // Canvert the image to the right format
        let data = img::convert_image(&background_image);

        // Copy image data to pixmap
        xcb::put_image_checked(conn, 2u8, pix, gcontext, w, h, 0, 0, 0, 32, &data)
            .request_check()
            .map_err(|e| ErrorKind::XError(format!("Unable to copy image to bg pixmap: {}", e)))?;
    }

    // Create new picture from pixmap
    let bg = conn.generate_id();
    xcb::render::create_picture_checked(conn, bg, pix, format32, &[])
        .request_check()
        .map_err(|e| ErrorKind::XError(format!("Unable to create bg picture: {}", e)))?;

    // Free the unneeded pixmap
    xcb::free_pixmap_checked(conn, pix)
        .request_check()
        .map_err(|e| ErrorKind::XError(format!("Unable to free temporary bg pixmap: {}", e)))?;

    Ok(bg)
}
