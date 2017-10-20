use std::sync::Arc;
use error::*;
use xcb;

// Get the screen from an XCB Connection
pub fn screen(conn: &Arc<xcb::Connection>) -> Result<xcb::Screen> {
    conn.get_setup()
        .roots()
        .next()
        .ok_or_else(|| ErrorKind::XcbNoScreenError(()).into())
}
