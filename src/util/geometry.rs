// Geometry of the bar
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Geometry {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl Geometry {
    // Helper for creating a geometry without struct syntax
    pub fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        Geometry {
            x,
            y,
            width,
            height,
        }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}
