extern crate image;
extern crate xcb;

use std::sync::Arc;
use xcb::Connection;
use std::path::Path;
use image::Pixel;
use std::thread;

macro_rules! set_prop {
    ($conn:expr, $window:expr, $name:expr, @atom $value:expr) => {
        {
            let atom_atom = xcb::intern_atom($conn, true, $value)
                .get_reply().unwrap().atom();
            set_prop!($conn, $window, $name, &[atom_atom], "ATOM", 32);
        }
    };
    ($conn:expr, $window:expr, $name:expr, $data:expr) => {
        {
            set_prop!($conn, $window, $name, $data, "CARDINAL", 32);
        }
    };
    ($conn:expr, $window:expr, $name:expr, $data:expr, $type:expr, $size:expr) => {
        {
            let type_atom = xcb::intern_atom($conn, true, $type).get_reply().unwrap().atom();
            let property = xcb::intern_atom($conn, true, $name)
                .get_reply().unwrap().atom();
            xcb::change_property(
                $conn,
                xcb::PROP_MODE_REPLACE as u8,
                $window,
                property,
                type_atom,
                $size,
                $data);
        }
    };
}

pub fn create_window() {
    let conn = Arc::new(Connection::connect(None).unwrap().0);
    let setup = conn.get_setup();
    let screen = setup.roots().next().unwrap();

    let background = (255u32 << 16) + (0u32 << 8) + 255u32;

    let window = conn.generate_id();

    xcb::create_window(
        &conn,
        xcb::WINDOW_CLASS_COPY_FROM_PARENT as u8,
        window,
        screen.root(),
        3600,
        0,
        2560,
        30,
        0,
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &[
            (xcb::CW_BACK_PIXEL, background), // Default background color
            (
                xcb::CW_EVENT_MASK, // What kinds of events are we
                xcb::EVENT_MASK_EXPOSURE |       //   interested in
             xcb::EVENT_MASK_KEY_PRESS | xcb::EVENT_MASK_ENTER_WINDOW,
            ),
            (xcb::CW_OVERRIDE_REDIRECT, 0),
        ],
    );

    set_prop!(&conn, window, "_NET_WM_WINDOW_TYPE", @atom "_NET_WM_WINDOW_TYPE_DOCK");
    set_prop!(&conn, window, "_NET_WM_STATE", @atom "_NET_WM_STATE_STICKY");
    set_prop!(&conn, window, "_NET_WM_DESKTOP", &[-1]);
    set_prop!(
        &conn,
        window,
        "_NET_WM_NAME",
        "window_title".as_bytes(),
        "UTF8_STRING",
        8
    );
    set_prop!(
        &conn,
        window,
        "WM_NAME",
        "window_title".as_bytes(),
        "STRING",
        8
    );

    // Request the WM to manage our window.
    xcb::map_window(&conn, window);

    {
        let conn = conn.clone();
        thread::spawn(move || {
            let pixmap = conn.generate_id();
            xcb::create_pixmap_checked(&conn, 24u8, pixmap, window, 2560u16, 30u16)
                .request_check()
                .unwrap();

            let gcontext = conn.generate_id();
            xcb::create_gc_checked(&conn, gcontext, window, &[])
                .request_check()
                .unwrap();

            let mut image = image::open(&Path::new("image.png")).unwrap().to_rgba();
            for pixel in image.pixels_mut() {
                let channels = pixel.channels_mut();
                let tmp0 = channels[2].clone();
                let tmp2 = channels[0].clone();
                channels[0] = tmp0;
                channels[2] = tmp2;
            }
            let data = &image.into_raw()[0..307199];

            xcb::put_image_checked(
                &conn,
                xcb::IMAGE_FORMAT_Z_PIXMAP as u8,
                pixmap,
                gcontext,
                2560,
                30,
                0,
                0,
                0,
                24u8,
                &data,
            ).request_check()
                .unwrap();

            xcb::copy_area_checked(&conn, pixmap, window, gcontext, 0, 0, 0, 0, 2560, 30)
                .request_check()
                .unwrap();

            conn.flush();
        });
    }

    loop {
        if let Some(event) = conn.wait_for_event() {
            let r = event.response_type();
            if r == xcb::EXPOSE {
                // Here we can send a request to all components to copy their area again
            }
        }
    }
}
