/// Width of a component.
///
/// This can override the width set by text or background. It can also be used to impose restraints
/// on the component's size.
///
/// # Examples
///
/// ```rust
/// use leechbar::Width;
///
/// // Width with min and max restrictions
/// let width = Width::new()
///                   .ignore_background()
///                   .min(100)
///                   .max(300);
///
/// // No width restrictions
/// let width = Width::new();
/// ```
#[derive(Copy, Clone, Default)]
pub struct Width {
    pub(crate) fixed: Option<u16>,
    pub(crate) min: u16,
    pub(crate) max: u16,
    pub(crate) ignore_background: bool,
    pub(crate) ignore_foreground: bool,
}

impl Width {
    /// Create a new width without any size restrictions.
    pub fn new() -> Self {
        Self {
            fixed: None,
            min: 0,
            max: ::std::u16::MAX,
            ignore_foreground: false,
            ignore_background: false,
        }
    }

    /// Set the component to a fixed with. This overrides min, max, background and text width.
    pub fn fixed(mut self, fixed: u16) -> Self {
        self.fixed = Some(fixed);
        self
    }

    /// Set the minimum width of a component.
    pub fn min(mut self, min: u16) -> Self {
        self.min = min;
        self
    }

    /// Set the maximum width of a component.
    pub fn max(mut self, max: u16) -> Self {
        self.max = max;
        self
    }

    /// When this is set, the width of the background is ignored.
    /// It is useful if you want to fit a background image to the width of the text.
    pub fn ignore_background(mut self) -> Self {
        self.ignore_background = true;
        self
    }

    /// When this is set, the width of the foreground is ignored.
    /// It is useful if you want to fit text to the width of the background. This will usually
    /// lead to text being cut off.
    pub fn ignore_foreground(mut self) -> Self {
        self.ignore_foreground = true;
        self
    }
}
