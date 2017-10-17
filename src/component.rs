use image::DynamicImage;
use std::time::Duration;
use util;

/// Alignment inside a container.
#[derive(Clone, Copy)]
pub enum Alignment {
    LEFT,
    CENTER,
    RIGHT,
}

impl Alignment {
    // Calculate the x-offset of a component based on its alignment
    pub(crate) fn x_offset(&self, comp_width: u16, width: u16) -> i16 {
        match *self {
            Alignment::LEFT => 0,
            Alignment::CENTER => (f64::from(comp_width) / 2. - f64::from(width) / 2.) as i16,
            Alignment::RIGHT => (comp_width - width) as i16,
        }
    }

    // Calculate the next id for a component
    pub(crate) fn id(&self, component_ids: &mut [u32; 3]) -> u32 {
        let index = match *self {
            Alignment::LEFT => 0,
            Alignment::CENTER => 1,
            Alignment::RIGHT => 2,
        };

        let return_val = component_ids[index];
        component_ids[index] += 3;
        return_val
    }
}

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
pub struct Text {
    pub(crate) content: String,
    pub(crate) font: Option<String>,
    pub(crate) color: Option<(f64, f64, f64, f64)>,
    pub(crate) alignment: Alignment,
    pub(crate) yoffset: Option<f64>,
}

impl Text {
    /// Create a new Text.
    pub fn new<T: Into<String>>(content: T) -> Self {
        Text {
            content: content.into(),
            font: None,
            color: None,
            alignment: Alignment::CENTER,
            yoffset: None,
        }
    }

    /// Set the font of the text.
    ///
    /// **Default:** Bar font.
    pub fn font<T: Into<String>>(mut self, font: T) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Set the foreground color of the text.
    ///
    /// **Default:** Bar foreground color.
    pub fn color(mut self, red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        self.color = Some((
            f64::from(red) / 255.,
            f64::from(green) / 255.,
            f64::from(blue) / 255.,
            f64::from(alpha) / 255.,
        ));
        self
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

/// Background of a component.
///
/// This is used to configure image- and color-based component backgrounds.
///
/// # Examples
///
/// ```rust
/// use leechbar::{Alignment, Background};
///
/// let bg = Background::new_color(255, 0, 255, 255)
///                     .alignment(Alignment::CENTER);
/// ```
#[derive(Clone)]
pub struct Background {
    pub(crate) color: Option<u32>,
    pub(crate) image: Option<DynamicImage>,
    pub(crate) alignment: Alignment,
}

impl Background {
    /// Create a background from an image.
    pub fn new_image(image: DynamicImage) -> Self {
        Background {
            image: Some(image),
            color: None,
            alignment: Alignment::CENTER,
        }
    }

    /// Create a background from a color.
    pub fn new_color(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Background {
            image: None,
            color: Some(util::color(red, green, blue, alpha)),
            alignment: Alignment::CENTER,
        }
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
    pub(crate) ignore_text: bool,
}

impl Width {
    /// Create a new width without any size restrictions.
    pub fn new() -> Self {
        Self {
            fixed: None,
            min: 0,
            max: ::std::u16::MAX,
            ignore_text: false,
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

    /// When this is set, the width of the text is ignored.
    /// It is useful if you want to fit text to the width of the background. This will usually
    /// lead to text being cut off.
    pub fn ignore_text(mut self) -> Self {
        self.ignore_text = true;
        self
    }
}

/// Trait for creating custom components
///
/// This trait is used for the [`Bar::add`] method. You can use it to implement custom components
/// that change at runtime. Each method takes `&mut self` and is called whenever the component
/// redraws, this allows mutating the struct of the component at runtime.
///
/// # Examples
///
/// ```rust
/// use leechbar::{Component, Text, Background, Alignment, Width};
/// use std::time::Duration;
///
/// struct MyComponent;
///
/// // You can define your own custom components like this
/// impl Component for MyComponent {
///     // No background image
///     fn background(&mut self) -> Option<Background> {
///         None
///     }
///
///     // Print "Hello, World!" as text
///     fn text(&mut self) -> Option<Text> {
///         Some(Text::new(String::from("Hello, World")))
///     }
///
///     // Put this element at the center of the bar
///     fn alignment(&mut self) -> Alignment {
///         Alignment::CENTER
///     }
///
///     // Do this only once
///     fn timeout(&mut self) -> Option<Duration> {
///         None
///     }
///
///     // No width restrictions
///     fn width(&mut self) -> Width {
///         Width::new()
///     }
///
///     // Ignore all events
///     fn event(&mut self) {}
/// }
///
/// // Create a new component
/// let component = MyComponent;
/// ```
///
/// [`Bar::add`]: struct.Bar.html#method.add
pub trait Component {
    /// The background of the component.
    /// Use `None` for no background.
    fn background(&mut self) -> Option<Background>;
    /// The alignment of the component.
    fn alignment(&mut self) -> Alignment;
    /// The text of the component.
    /// Use `None` for no text.
    fn text(&mut self) -> Option<Text>;
    /// The width of the component.
    fn width(&mut self) -> Width;
    /// The polling rate for this component. This is the time between redrawing the component.
    /// Use `None` for drawing this component once.
    fn timeout(&mut self) -> Option<Duration>;
    /// X.Org events. This is not implemented yet.
    fn event(&mut self); // TODO: Create event type
}
