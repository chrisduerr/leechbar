use component::background::Background;
use component::foreground::Foreground;
use component::alignment::Alignment;
use util::geometry::Geometry;
use util::color::Color;
use std::sync::Arc;
use chan::Sender;
use event::Event;
use bar::Bar;
use error::*;
use xcb;

#[derive(PartialEq, Clone)]
pub struct BarComponentCache {
    yoffset: i16,
    pictures: Vec<u32>,
    color: Option<Color>,
    alignment: Alignment,
}

impl BarComponentCache {
    // Create an empty cache
    pub fn new() -> Self {
        Self {
            yoffset: 0,
            color: None,
            pictures: Vec::new(),
            alignment: Alignment::CENTER,
        }
    }

    // Create a cache form a background
    pub fn new_bg(background: &Background) -> Self {
        Self {
            yoffset: 0,
            color: background.color,
            alignment: background.alignment,
            pictures: background.images.iter().map(|i| i.arc.xid).collect(),
        }
    }

    // Create a cache from a foreground
    pub fn new_fg(foreground: &Foreground) -> Self {
        Self {
            color: None,
            alignment: foreground.alignment,
            // Should always be `Some`, just making sure
            yoffset: foreground.yoffset.unwrap_or(0),
            pictures: vec![foreground.text.as_ref().map(|t| t.arc.xid)]
                .iter()
                .filter_map(|x| *x)
                .collect(),
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
    pub picture: u32,
    pub geometry: Geometry,
    pub interrupt: Option<Sender<Event>>,
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
            interrupt: None,
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
        xtry!(
            create_pixmap_checked,
            &bar.conn,
            32,
            tmp_pix,
            bar.window,
            w,
            h
        );

        // Clear content of pixmap
        let rect = &[xcb::Rectangle::new(0, 0, w, h)];
        xtry!(
            poly_fill_rectangle_checked,
            &bar.conn,
            tmp_pix,
            bar.gcontext,
            rect
        );

        // Create picture for intermediate pixmap
        let tmp_pict = bar.conn.generate_id();
        xtry!(@render create_picture_checked, &bar.conn, tmp_pict, tmp_pix, bar.format32, &[]);

        // Copy over background
        let op = xcb::render::PICT_OP_OVER as u8;

        // Copy the background of the bar to that picture
        let bg = bar.background;
        xtry!(@render composite_checked, &bar.conn, op, bg, 0, tmp_pict, x, 0, 0, 0, 0, 0, w, h);

        // Copy the component to the temporary picture
        let pict = self.picture;
        xtry!(@render composite_checked, &bar.conn, op, pict, 0, tmp_pict, 0, 0, 0, 0, 0, 0, w, h);

        bar.composite_picture(tmp_pict, 0, x, w, h)?;

        // Free the picture and pixmap
        xcb::free_pixmap(&bar.conn, tmp_pix);
        xcb::render::free_picture(&bar.conn, tmp_pict);
        Ok(())
    }
}
