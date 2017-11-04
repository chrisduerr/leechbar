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
/// use leechbar::{Foreground, Text, BarBuilder, Alignment};
///
/// let bar = BarBuilder::new().spawn().unwrap();
/// let text = Text::new(&bar, "Hello, World", None, None).unwrap();
/// let fg = Foreground::new()
///                     .text(text)
///                     .yoffset(3)
///                     .alignment(Alignment::RIGHT);
/// ```
#[derive(Clone)]
pub struct Foreground {
    pub(crate) text: Option<Text>,
    pub(crate) alignment: Alignment,
    pub(crate) yoffset: Option<i16>,
}

impl Foreground {
    /// Create a new empty Foreground.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::Foreground;
    ///
    /// let fg = Foreground::new();
    /// ```
    pub fn new() -> Self {
        Foreground {
            text: None,
            yoffset: None,
            alignment: Alignment::CENTER,
        }
    }

    /// Set the text of the foreground.
    ///
    /// **Default:** No text.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use leechbar::{Foreground, Text, BarBuilder};
    ///
    /// let bar = BarBuilder::new().spawn().unwrap();
    /// let text = Text::new(&bar, "Text :)", None, None).unwrap();
    /// let fg = Foreground::new().text(text);
    /// ```
    pub fn text(mut self, text: Text) -> Self {
        self.text = Some(text);
        self
    }

    /// Set the alignment of the text inside the component.
    ///
    /// **Default:** [`Alignment::CENTER`](enum.Alignment.html#variant.CENTER)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::{Foreground, Alignment};
    ///
    /// let fg = Foreground::new().alignment(Alignment::RIGHT);
    /// ```
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Offset the text vertically. Increasing this offset, moves the text down from the center.
    ///
    /// **Default:** Bar's vertical text offset.
    ///
    /// ```rust
    /// use leechbar::Foreground;
    ///
    /// let fg = Foreground::new().yoffset(-3);
    /// ```
    pub fn yoffset(mut self, yoffset: i16) -> Self {
        self.yoffset = Some(yoffset);
        self
    }
}

impl From<Text> for Foreground {
    fn from(text: Text) -> Foreground {
        Foreground {
            yoffset: None,
            text: Some(text),
            alignment: Alignment::CENTER,
        }
    }
}

impl Default for Foreground {
    fn default() -> Self {
        Self::new()
    }
}
