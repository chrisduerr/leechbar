use geometry::Geometry;
use std::sync::Arc;
use xcb;

// Picture with known size
pub struct Picture {
    pub(crate) conn: Arc<xcb::Connection>,
    pub(crate) geometry: Geometry,
    pub(crate) xid: u32,
}

// Drop picture when it goes out of scope
impl Drop for Picture {
    fn drop(&mut self) {
        xcb::render::free_picture(&self.conn, self.xid);
    }
}
