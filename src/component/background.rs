use component::alignment::Alignment;
use component::img::Image;
use util::color::Color;

/// Background of a component.
///
/// This is used to configure image- and color-based component backgrounds.
///
/// # Examples
///
/// ```rust
/// use leechbar::{Alignment, Background, Color};
///
/// let bg = Background::new()
///                     .color(Color::new(255, 0, 255, 255));
/// ```
#[derive(Clone)]
pub struct Background {
    pub(crate) images: Vec<Image>,
    pub(crate) color: Option<Color>,
    pub(crate) alignment: Alignment,
}

impl Background {
    /// Create a new empty background
    ///
    /// ```rust
    /// use leechbar::Background;
    ///
    /// let bg = Background::new();
    /// ```
    pub fn new() -> Self {
        Self {
            color: None,
            images: Vec::new(),
            alignment: Alignment::CENTER,
        }
    }

    /// Add an image to the background. This can be called multiple times to layer images above
    /// each other.
    ///
    /// ```rust,no_run
    /// # extern crate leechbar;
    /// extern crate image;
    /// use leechbar::{Background, Image, BarBuilder};
    ///
    /// # fn main() {
    /// // Create the bar
    /// let bar = BarBuilder::new().spawn().unwrap();
    ///
    /// // Convert our images
    /// let img1 = image::open("my_image2").unwrap();
    /// let img2 = image::open("my_image2").unwrap();
    /// let ximg1 = Image::new(&bar, &img1).unwrap();
    /// let ximg2 = Image::new(&bar, &img2).unwrap();
    ///
    /// // Create a background with our images
    /// let bg = Background::new().image(ximg1).image(ximg2);
    /// # }
    /// ```
    pub fn image<T: Into<Image>>(mut self, image: T) -> Self {
        self.images.push(image.into());
        self
    }

    /// Set the background color.
    ///
    /// ```rust
    /// use leechbar::{Background, Color};
    ///
    /// let bg = Background::new().color(Color::new(255, 0, 255, 255));
    /// ```
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl From<Image> for Background {
    fn from(image: Image) -> Background {
        Background {
            color: None,
            images: vec![image],
            alignment: Alignment::CENTER,
        }
    }
}

impl From<Color> for Background {
    fn from(color: Color) -> Background {
        Background {
            color: Some(color),
            images: Vec::new(),
            alignment: Alignment::CENTER,
        }
    }
}

impl Default for Background {
    fn default() -> Self {
        Self::new()
    }
}
