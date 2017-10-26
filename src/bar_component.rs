use background::Background;
use foreground::Foreground;
use alignment::Alignment;
use geometry::Geometry;
use std::sync::Arc;
use color::Color;
use bar::Bar;
use error::*;
use xcb;

#[derive(PartialEq, Clone, Copy)]
pub struct BarComponentCache {
    picture: u32,
    yoffset: i16,
    color: Option<Color>,
    alignment: Alignment,
}

impl BarComponentCache {
    // Create an empty cache
    pub fn new() -> Self {
        Self {
            picture: 0,
            yoffset: 0,
            color: None,
            alignment: Alignment::CENTER,
        }
    }

    // Create a cache form a background
    pub fn new_bg(background: &Background) -> Self {
        Self {
            yoffset: 0,
            color: background.color,
            alignment: background.alignment,
            picture: background.image.as_ref().map_or(0, |i| i.arc.xid),
        }
    }

    // Create a cache from a foreground
    pub fn new_fg(foreground: &Foreground) -> Self {
        Self {
            color: None,
            alignment: foreground.alignment,
            // Should always be `Some`, just making sure
            yoffset: foreground.yoffset.unwrap_or(0),
            picture: foreground.text.as_ref().map_or(0, |t| t.arc.xid),
        }
    }
}

impl Default for BarComponentCache {
    fn default() -> Self {
        BarComponentCache::new()
    }
}

// A component currently stored in the bar
pub struct BarComponent {
    pub id: u32,
    pub dirty: bool,
    pub picture: u32,
    pub geometry: Geometry,
    pub bg_cache: BarComponentCache,
    pub fg_cache: BarComponentCache,
}

impl BarComponent {
    // Creates a new component
    pub fn new(id: u32, conn: &Arc<xcb::Connection>) -> Self {
        let picture = conn.generate_id();
        BarComponent {
            id,
            picture,
            dirty: false,
            geometry: Geometry::default(),
            bg_cache: BarComponentCache::new(),
            fg_cache: BarComponentCache::new(),
        }
    }

    // Update a component cached by the bar
    pub fn set_geometry(&mut self, geometry: Geometry) {
        self.geometry = geometry;
    }

    // Redraw a component
    // Copies the pixmap to the window
    pub fn redraw(&self, bar: &Bar) -> Result<()> {
        // Shorten geometry names
        let (w, h, x) = (self.geometry.width, self.geometry.height, self.geometry.x);

        // Create an intermediate pixmap
        let tmp_pix = bar.conn.generate_id();
        xtry!(create_pixmap_checked, &bar.conn, 32, tmp_pix, bar.window, w, h);

        // Clear content of pixmap
        let rect = &[xcb::Rectangle::new(0, 0, w, h)];
        xtry!(poly_fill_rectangle_checked, &bar.conn, tmp_pix, bar.gcontext, rect);

        // Create picture for intermediate pixmap
        let tmp_pict = bar.conn.generate_id();
        xtry!(@render create_picture_checked, &bar.conn, tmp_pict, tmp_pix, bar.format32, &[]);

        // Copy over background
        let op = xcb::render::PICT_OP_OVER as u8;

        // Copy teh background of the bar to that picture
        xtry!(@render composite_checked, &bar.conn, op, bar.background, 0, tmp_pict, x, 0, 0, 0, 0, 0, w, h);

        // Copy the component to the temporary picture
        xtry!(@render composite_checked, &bar.conn, op, self.picture, 0, tmp_pict, 0, 0, 0, 0, 0, 0, w, h);

        bar.composite_picture(tmp_pict, 0, x, w, h)?;

        // Free the picture and pixmap
        xcb::free_pixmap(&bar.conn, tmp_pix);
        xcb::render::free_picture(&bar.conn, tmp_pict);
        Ok(())
    }
}
