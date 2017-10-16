use component::{Alignment, Background, Component, Text, Width};
use cairo::{Context, Format, ImageSurface, Surface};
use pango::{FontDescription, Layout, LayoutExt};
use xcb::{self, Rectangle, Screen, Visualtype};
use image::{DynamicImage, GenericImage};
use bar_component::BarComponent;
use pangocairo::CairoContextExt;
use geometry::Geometry;
use std::sync::Arc;
use cairo_sys;
use error::*;
use std::cmp;
use bar::Bar;
use util;

// Renders the state of a component to the bar
pub fn render(
    bar: &Bar,
    component: &mut Component,
    mut font: FontDescription,
    id: u32,
) -> Result<()> {
    // Shorten a few properties for the massive xcb methods
    let (conn, gc, win) = (&bar.conn, bar.gcontext, bar.window);

    // Get new text and background from component
    let background = component.background();
    let mut text = component.text();
    let width = component.width();

    // Override the global font and color
    if let Some(ref mut text) = text {
        // Override bar font if component font is some
        if let Some(ref font_override) = text.font {
            font = FontDescription::from_string(font_override);
        }

        // Use bar foreground if component foreground is none
        if text.color.is_none() {
            text.color = Some(bar.color);
        }
    }

    // Calculate width and height of element
    let h = bar.geometry.height;
    let w = calculate_width(bar, width, &background, &text, &font);

    // Create pixmap
    let pix = conn.generate_id();
    xtry!(create_pixmap_checked, conn, 32, pix, win, w, h);
    xtry!(poly_fill_rectangle_checked, conn, pix, gc, &[Rectangle::new(0, 0, w, h)]);

    {
        // Lock the components
        let mut components = bar.components.lock().unwrap();
        // Get the X offset of the item
        let mut x = xoffset_by_id(&components, id, w, bar.geometry.width);

        // Add background to pixmap
        if let Some(background) = background {
            let align = background.alignment;
            let image = &background.image;
            let color = background.color;
            render_background(bar, pix, w, h, align, color, image)?;
        }

        // Add text to pixmap
        if let Some(text) = text {
            let screen = util::screen(conn)?;
            render_text(conn, &screen, pix, w, h, &font, &text)?;
        }

        // TODO: If width did not change, just clear and redraw this single component

        // Get all components that need to be redrawn
        components.sort_by(|a, b| a.id.cmp(&b.id));
        let components = components
            .iter_mut()
            .filter(|c| (c.id % 3 != 0 || c.id >= id) && c.id % 3 == id % 3)
            .collect::<Vec<&mut BarComponent>>();

        // Clear the difference to old components
        let comp_index = components.binary_search_by_key(&id, |c| c.id).unwrap_or(0);
        let width_change = i32::from(components[comp_index].geometry.width) - i32::from(w);
        if width_change > 0 {
            clear_old_components(bar, &(*components), x, width_change as i16)?;
        }

        // Redraw all selected components
        for component in components {
            // Old rectangle for clearing bar
            let (w, h) = if component.id == id {
                // Update picture with the new pixmap
                let pict = component.picture;
                xcb::render::free_picture(conn, pict);
                xtry!(@render create_picture_checked, conn, pict, pix, bar.format32, &[]);

                // Free the pixmap after picture has been created
                xcb::free_pixmap(conn, pix);

                // Return component dimensions
                (w, h)
            } else {
                (component.geometry.width, component.geometry.height)
            };

            // Update the component
            component.set_geometry(Geometry::new(x, 0, w, h));

            // Redraw the component
            if w > 0 && h > 0 {
                component.redraw(bar)?;
                x += w as i16;
            }
        }
    }

    // Flush XCB Connection
    conn.flush();

    Ok(())
}

// Render the background image/color
fn render_background(
    bar: &Bar,
    pix: u32,
    w: u16,
    h: u16,
    alignment: Alignment,
    color: Option<u32>,
    image: &Option<DynamicImage>,
) -> Result<()> {
    // Shorten bar variable names
    let (conn, gc) = (&bar.conn, bar.gcontext);

    if let Some(color) = color {
        // Create a GC with the color
        let col_gc = conn.generate_id();
        xtry!(
            create_gc_checked,
            conn,
            col_gc,
            pix,
            &[(xcb::ffi::xproto::XCB_GC_FOREGROUND, color)]
        );

        // Fill the pixmap with the GC color
        xtry!(poly_fill_rectangle_checked, conn, pix, col_gc, &[Rectangle::new(0, 0, w, h)]);

        // Free gc after filling the rectangle
        xcb::free_gc(conn, col_gc);
    }

    // Copy image if there is an image
    if let Some(ref image) = *image {
        // Convert image to raw pixels
        let data = util::convert_image(image);

        // Get width and height of the image
        let iw = image.width() as u16;
        let ih = image.height() as u16;

        // Get X position
        let x = alignment.x_offset(w, iw);

        // Put image on pixmap
        xtry!(put_image_checked, conn, 2u8, pix, gc, iw, ih, x, 0, 0, 32, &data);
    }

    Ok(())
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

// Render text to a pixmap
fn render_text(
    conn: &Arc<xcb::Connection>,
    screen: &xcb::Screen,
    pixmap: u32,
    width: u16,
    height: u16,
    font: &FontDescription,
    text: &Text,
) -> Result<()> {
    // Create an xcb surface
    let mut visualtype = find_visualtype32(screen).ok_or_else(|| ErrorKind::ScreenDepthError(()))?;
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
    let color = text.color.unwrap(); // This is always Some
    context.set_source_rgba(color.0, color.1, color.2, color.3);

    // Center text horizontally and vertically
    let (text_width, text_height) = layout.get_pixel_size();
    let text_y = (f64::from(height) - f64::from(text_height)) / 2.;
    let text_x = f64::from(text.alignment.x_offset(width, text_width as u16));
    context.move_to(text_x, text_y + text.yoffset);

    // Display text
    context.show_pango_layout(&layout);

    Ok(())
}

// Clear the old area before redrawing
fn clear_old_components(
    bar: &Bar,
    components: &[&mut BarComponent],
    new_start: i16,
    width_change: i16,
) -> Result<()> {
    // Bar shorthands
    let bar_height = bar.geometry.height;

    // Get old start x
    let old_width_all = components.iter().map(|c| c.geometry.width).sum::<u16>() as i16;
    let old_start = components[0].geometry.x;

    // Redraw from old_x to new_x
    if old_start < new_start {
        let width = (new_start - old_start) as u16;
        bar.composite_picture(bar.background, old_start, old_start, width, bar_height)?;
    }

    // Get the old end x and new end x
    let old_end = old_start + old_width_all;
    let new_end = old_end - width_change;

    if old_end > new_end {
        let width = (old_end - new_end) as u16;
        bar.composite_picture(bar.background, new_end, new_end, width, bar.geometry.height)?;
    }

    Ok(())
}

// Calculate the width of a component
fn calculate_width(
    bar: &Bar,
    width: Width,
    background: &Option<Background>,
    text: &Option<Text>,
    font: &FontDescription,
) -> u16 {
    // Just return fixed if it's some
    if let Some(fixed) = width.fixed {
        return fixed;
    }

    // Start with min which defaults to 0
    let mut w = width.min;

    // Set to background width if it isn't smaller than min
    if let Some(ref background) = *background {
        if let Some(ref image) = background.image {
            // Check if bg width should be ignored
            if !width.ignore_background {
                w = cmp::max(w, image.width() as u16);
            }
        }
    }

    // Set to text width if it isn't smaller than min
    if let Some(ref text) = *text {
        // Check if text width should be ignored
        if !width.ignore_text {
            let text_width = text_width(&text.content, font).unwrap_or(0);
            w = cmp::max(w, text_width);
        }
    }

    // Make sure it's not bigger than the whole bar
    w = cmp::min(w, bar.geometry.width);

    // Make sure it's not bigger than max
    w = cmp::min(w, width.max);

    w
}

// Get the width text will have with the specified font
pub fn text_width(text: &str, font: &FontDescription) -> Result<(u16)> {
    // Create a dummy surface and context
    let surface = ImageSurface::create(Format::ARgb32, 0, 0)
        .map_err(|e| format!("Unable to create dummy layout for font size: {:?}", e))?;
    let context = Context::new(&surface);

    // Create the layout
    let layout = layout(&context, text, font);

    // Get the width of the text
    let width = layout.get_pixel_size().0;

    Ok(width as u16)
}

// Create a layout with the font and text
fn layout(context: &Context, text: &str, font: &FontDescription) -> Layout {
    let layout = context.create_pango_layout();
    layout.set_text(text);
    layout.set_font_description(font);
    layout
}

// Get the first available visualtype with 32 bit depth
fn find_visualtype32<'s>(screen: &Screen<'s>) -> Option<Visualtype> {
    for depth in screen.allowed_depths() {
        if depth.depth() == 32 {
            let visual = depth.visuals().next();
            if let Some(visual) = visual {
                return Some(visual);
            }
        }
    }
    None
}
