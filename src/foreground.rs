use alignment::Alignment;
use text::Text;

/// Text of a component.
///
/// This is used for displaying text on the bar.
///
/// # Examples
///
/// ```rust
/// use leechbar::{Text, Alignment};
///
/// let text = Text::new("Hello, World!")
///                 .font("Fira Sans Medium 11")
///                 .color(255, 0, 255, 255)
///                 .alignment(Alignment::LEFT);
/// ```
#[derive(Clone)]
pub struct Foreground {
    pub(crate) text: Text,
    pub(crate) alignment: Alignment,
    pub(crate) yoffset: Option<f64>,
}

impl Foreground {
    /// Create a new Foreground.
    pub fn new<T: Into<Text>>(text: T) -> Self {
        Foreground {
            yoffset: None,
            text: text.into(),
            alignment: Alignment::CENTER,
        }
    }

    /// Set the alignment of the text inside the component.
    ///
    /// **Default:** [`Alignment::CENTER`](enum.Alignment.html#variant.CENTER)
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Offset the text vertically. Increasing this offset, moves the text down from the center.
    ///
    /// **Default:** Bar's vertical text offset.
    pub fn yoffset(mut self, yoffset: f64) -> Self {
        self.yoffset = Some(yoffset);
        self
    }
}
