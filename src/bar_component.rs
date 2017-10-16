use geometry::Geometry;
use std::sync::Arc;
use bar::Bar;
use error::*;
use xcb;

// A component currently stored in the bar
pub struct BarComponent {
    pub id: u32,
    pub picture: u32,
    pub geometry: Geometry,
}

impl BarComponent {
    // Creates a new component
    pub fn new(id: u32, conn: &Arc<xcb::Connection>) -> Self {
        let picture = conn.generate_id();
        BarComponent {
            geometry: Geometry::default(),
            picture,
            id,
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
        Ok(())
    }
}
