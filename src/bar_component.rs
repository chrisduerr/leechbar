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
        let (w, h, x) = (self.geometry.width, self.geometry.height, self.geometry.x);
        bar.composite_picture(self.picture, 0, x, w, h)?;
        Ok(())
    }

    // Clear the area of this component
    // This should be called before updating it
    pub fn clear(&self, bar: &Bar) -> Result<()> {
        let (w, h, x) = (self.geometry.width, self.geometry.height, self.geometry.x);
        if bar.background != 0 {
            // Copy image if background exists
            bar.composite_picture(bar.background, x, x, w, h)?;
        } else {
            // Clear rectangle if there is no background image
            xtry!(clear_area_checked, &bar.conn, false, bar.window, x, 0, w, h);
        }

        Ok(())
    }
}
