use alignment::Alignment;
use color::Color;
use img::Image;

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
///                     .color(Color::new(255, 0, 255, 255))
///                     .alignment(Alignment::CENTER);
/// ```
#[derive(Clone)]
pub struct Background {
    pub(crate) color: Option<Color>,
    pub(crate) image: Option<Image>,
    pub(crate) alignment: Alignment,
}

impl Background {
    /// Create a new empty background
    pub fn new() -> Self {
        Self {
            image: None,
            color: None,
            alignment: Alignment::CENTER,
        }
    }

    /// Set the background image.
    pub fn image<T: Into<Image>>(mut self, image: T) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Set the background color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the alignment of the background image.
    ///
    /// This does nothing for a [`new_color`](#method.new_color) background.
    ///
    /// **Default:** [`Alignment::CENTER`](enum.Alignment.html#variant.CENTER)
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl From<Image> for Background {
    fn from(image: Image) -> Background {
        Background {
            color: None,
            image: Some(image),
            alignment: Alignment::CENTER,
        }
    }
}

impl From<Color> for Background {
    fn from(color: Color) -> Background {
        Background {
            image: None,
            color: Some(color),
            alignment: Alignment::CENTER,
        }
    }
}

impl Default for Background {
    fn default() -> Self {
        Self::new()
    }
}
