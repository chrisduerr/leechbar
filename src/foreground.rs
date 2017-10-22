use alignment::Alignment;
use text::Text;

/// Foreground of a component.
///
/// The foreground of a component. This is used for setting the text, the text alignment and the
/// vertical text offset.
///
/// # Examples
///
/// ```rust,no_run
/// use leechbar::{Foreground, Text, BarBuilder};
///
/// let bar = BarBuilder::new().spawn().unwrap();
/// let text = Text::new(&bar, "Hello, World", None, None).unwrap();
/// let fg = Foreground::new(text).yoffset(3.5);
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
