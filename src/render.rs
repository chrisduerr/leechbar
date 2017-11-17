use component::bar_component::{BarComponent, BarComponentCache};
use component::foreground::Foreground;
use component::background::Background;
use component::alignment::Alignment;
use component::picture::Picture;
use util::geometry::Geometry;
use component::width::Width;
use xcb::{self, Rectangle};
use component::Component;
use util::color::Color;
use std::sync::Arc;
use error::*;
use std::cmp;
use bar::Bar;

// Renders the state of a component to the bar
pub fn render(bar: &Bar, component: &mut Component, id: u32) -> Result<()> {
    // Shorten a few properties for the massive xcb methods
    let conn = &bar.conn;

    // Get new text and background from component
    let width = component.width();
    let background = component.background();
    let foreground = component.foreground();

    // Calculate width and height of element
    let h = bar.geometry.height;
    let w = calculate_width(bar, width, &background, &foreground);

    {
        // Lock the components
        let mut components = bar.components.lock().unwrap();
        // Get the X offset of the item
        let mut x = xoffset_by_id(&components, id, w, bar.geometry.width);

        // Get all components that need to be redrawn
        components.sort_by(|a, b| a.id.cmp(&b.id));
        let mut components = components
            .iter_mut()
            .filter(|c| (c.id % 3 != 0 || c.id >= id) && c.id % 3 == id % 3)
            .collect::<Vec<&mut BarComponent>>();

        // Get the index of the current component
        let comp_index = components.binary_search_by_key(&id, |c| c.id).unwrap_or(0);

        // Update if background or foreground changed
        let new_fg_cache = BarComponentCache::new_fg(&foreground);
        let new_bg_cache = BarComponentCache::new_bg(&background);
        let old_fg_cache = components[comp_index].fg_cache;
        let old_bg_cache = components[comp_index].bg_cache;
        let old_width = components[comp_index].geometry.width;
        let old_height = components[comp_index].geometry.height;
        if new_bg_cache != old_bg_cache || new_fg_cache != old_fg_cache || old_width != w
            || old_height != h
        {
            debug!("Recomposing {}…", id);
            update_picture(bar, &mut components[comp_index], &background, &foreground, w, h)?;
        }

        // Clear the difference to old components
        let width_change = i32::from(components[comp_index].geometry.width) - i32::from(w);
        if width_change > 0 {
            clear_old_components(bar, &(*components), x, width_change as i16)?;
        }

        // Redraw all selected components
        for component in components {
            // Old rectangle for clearing bar
            let (w, h) = if component.id == id {
                // Return component dimensions
                (w, h)
            } else {
                (component.geometry.width, component.geometry.height)
            };

            // Update the component
            component.set_geometry(Geometry::new(x, 0, w, h));

            // Don't redraw other components if width didn't change
            // Don't redraw empty components
            if w > 0 && h > 0 && (width_change != 0 || component.id == id) {
                // Redraw the component
                debug!("Redrawing {}…", component.id);
                component.redraw(bar)?;
            }
            x += w as i16;
        }
    }

    // Flush XCB Connection
    conn.flush();

    Ok(())
}

// Update the picture of a `BarComponent`
fn update_picture(
    bar: &Bar,
    component: &mut BarComponent,
    background: &Background,
    foreground: &Foreground,
    w: u16,
    h: u16,
) -> Result<()> {
    // Don't update the pixmap when it's empty
    if w == 0 || h == 0 {
        return Ok(());
    }

    // Shorten variable names
    let (conn, gc, win) = (&bar.conn, bar.gcontext, bar.window);

    // Create pixmap with empty background
    let pix = conn.generate_id();
    xtry!(create_pixmap_checked, conn, 32, pix, win, w, h);
    xtry!(poly_fill_rectangle_checked, conn, pix, gc, &[Rectangle::new(0, 0, w, h)]);

    // Free old picture
    let pict = component.picture;
    xcb::render::free_picture(conn, pict);

    // Create picture from pixmap
    xtry!(@render create_picture_checked, conn, pict, pix, bar.format32, &[]);

    // Render the background color
    if let Some(color) = background.color {
        render_color(bar, pix, w, h, color)?;
    }

    // Render the background image if it's not `None`
    if let Some(ref image) = background.image {
        render_picture(bar, pict, w, &image.arc, background.alignment, 0)?;
    }

    // Render the foreground text
    if let Some(ref text) = foreground.text {
        let yoffset = foreground.yoffset.unwrap_or(bar.text_yoffset);
        render_picture(bar, pict, w, &text.arc, foreground.alignment, yoffset)?;
    }

    // Free pixmap
    xcb::free_pixmap(conn, pix);

    Ok(())
}

// Render the a color to a pixmap
fn render_color(bar: &Bar, pix: u32, w: u16, h: u16, color: Color) -> Result<()> {
    // Shorten bar variable names
    let conn = &bar.conn;

    // Create a GC with the color
    let col_gc = conn.generate_id();
    xtry!(
        create_gc_checked,
        conn,
        col_gc,
        pix,
        &[(xcb::ffi::xproto::XCB_GC_FOREGROUND, color.into())]
    );

    // Fill the pixmap with the GC color
    xtry!(poly_fill_rectangle_checked, conn, pix, col_gc, &[Rectangle::new(0, 0, w, h)]);

    // Free gc after filling the rectangle
    xcb::free_gc(conn, col_gc);

    Ok(())
}

// Render picture over a picture
fn render_picture(
    bar: &Bar,
    tar_pict: u32,
    w: u16,
    src_pict: &Arc<Picture>,
    alignment: Alignment,
    yoffset: i16,
) -> Result<()> {
    // Shorten bar variable names
    let conn = &bar.conn;

    // Get width and height of the picture
    let pw = src_pict.geometry.width;
    let ph = src_pict.geometry.height;

    // Get X position
    let x = alignment.x_offset(w, pw);

    // Put image on pixmap
    let op = xcb::render::PICT_OP_OVER as u8;
    xtry!(@render composite_checked, conn, op, src_pict.xid, 0, tar_pict, 0, 0, 0, 0, x, yoffset, pw, ph);

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
    background: &Background,
    foreground: &Foreground,
) -> u16 {
    // Just return fixed if it's some
    if let Some(fixed) = width.fixed {
        return cmp::min(fixed, bar.geometry.width);
    }

    // Start with min which defaults to 0
    let mut w = width.min;

    // Set to background width if it isn't smaller than min
    if let Some(ref image) = background.image {
        // Check if bg width should be ignored
        if !width.ignore_background {
            w = cmp::max(w, image.arc.geometry.width);
        }
    }

    // Set to text width if it isn't smaller than min
    if let Some(ref text) = foreground.text {
        // Check if text width should be ignored
        if !width.ignore_foreground {
            w = cmp::max(w, text.arc.geometry.width);
        }
    }

    // Make sure it's not bigger than the whole bar
    w = cmp::min(w, bar.geometry.width);

    // Make sure it's not bigger than max
    w = cmp::min(w, width.max);

    w
}
