/// RGBA color structure.
#[derive(Copy, Clone, PartialEq)]
pub struct Color {
    pub(crate) red: u8,
    pub(crate) green: u8,
    pub(crate) blue: u8,
    pub(crate) alpha: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    // Change from 0..255 to 0..1
    pub fn as_fractions(&self) -> (f64, f64, f64, f64) {
        (
            f64::from(self.red) / 255.,
            f64::from(self.green) / 255.,
            f64::from(self.blue) / 255.,
            f64::from(self.alpha) / 255.,
        )
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> u32 {
        ((u32::from(color.alpha)) << 24) + ((u32::from(color.red)) << 16)
            + ((u32::from(color.green)) << 8) + u32::from(color.blue)
    }
}
