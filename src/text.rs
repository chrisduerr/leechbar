use cairo::{Context, Format, ImageSurface, Surface};
use pango::{FontDescription, Layout, LayoutExt, SCALE};
use pangocairo::CairoContextExt;
use xcb::{self, Screen, Visualtype};
use component::{Alignment, Text};
use std::sync::Arc;
use cairo_sys;
use error::*;

pub fn render_text(
    conn: &Arc<xcb::Connection>,
    screen: &xcb::Screen,
    pixmap: u32,
    width: u16,
    height: u16,
    font: &FontDescription,
    text: &Text,
) {
    // Create an xcb surface
    let mut visualtype = find_visualtype(screen).unwrap();
    let surface = unsafe {
        Surface::from_raw_full(cairo_sys::cairo_xcb_surface_create(
            (conn.get_raw_conn() as *mut cairo_sys::xcb_connection_t),
            pixmap,
            (&mut visualtype.base as *mut xcb::ffi::xcb_visualtype_t)
                as *mut cairo_sys::xcb_visualtype_t,
            i32::from(width),
            i32::from(height),
        ))
    };

    // Create context and layout for drawing text
    let context = Context::new(&surface);
    let layout = layout(&context, &text.content, font);

    // Set font color
    context.set_source_rgb(0., 0., 0.);

    // Center text horizontally and vertically
    let text_height = f64::from(font.get_size()) / f64::from(SCALE);
    let text_bottom = (f64::from(height) / 2. + text_height / 2.
        - f64::from(layout.get_baseline()) / f64::from(SCALE))
        .floor() - 1.;
    let text_width = f64::from(text_size(&text.content, font).unwrap().0);
    let text_left = match text.alignment {
        Alignment::LEFT => 0.,
        Alignment::CENTER => (f64::from(width) / 2. - text_width / 2.).floor() - 1.,
        Alignment::RIGHT => f64::from(width) - text_width,
    };
    context.move_to(text_left, text_bottom);

    // Display text
    context.show_pango_layout(&layout);
}

// Get the size text will have with the specified font
pub fn text_size(text: &str, font: &FontDescription) -> Result<(u16, u16)> {
    // Create a dummy surface and context
    let surface = ImageSurface::create(Format::ARgb32, 0, 0).map_err(|e| {
        format!("Unable to create dummy layout for font size: {:?}", e)
    })?;
    let context = Context::new(&surface);

    // Create the layout
    let layout = layout(&context, text, font);

    // Get the width and height of the text
    let (height, width) = layout.get_pixel_size();

    Ok((height as u16, width as u16))
}

// Create a layout with the font and text
fn layout(context: &Context, text: &str, font: &FontDescription) -> Layout {
    let layout = context.create_pango_layout();
    layout.set_text(text);
    layout.set_font_description(font);
    layout
}

// Get the default visualtype of a screen
fn find_visualtype<'s>(screen: &Screen<'s>) -> Option<Visualtype> {
    for depth in screen.allowed_depths() {
        for visual in depth.visuals() {
            if visual.visual_id() == screen.root_visual() {
                return Some(visual);
            }
        }
    }
    None
}
