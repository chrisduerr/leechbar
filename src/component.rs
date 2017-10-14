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
}

/// Alignment and order of a component.
///
/// The alignment controls the position inside the bar (left/center/right).
///
/// The ordinal is used to order the components of the bar. A left-aligned component with ordinal 0,
/// will be left of a left-aligned component with ordinal 1.
#[derive(Clone, Copy)]
pub struct ComponentPosition {
    alignment: Alignment,
    ordinal: u32,
}

impl ComponentPosition {
    /// Create a new component position.
    pub fn new(alignment: Alignment, ordinal: u32) -> Self {
        ComponentPosition { alignment, ordinal }
    }

    // Use the position and alignment of the item to get a unique id.
    pub(crate) fn unique_id(&self) -> u32 {
        match self.alignment {
            Alignment::LEFT => self.ordinal * 3,
            Alignment::CENTER => self.ordinal * 3 + 1,
            Alignment::RIGHT => self.ordinal * 3 + 2,
        }
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
/// ```
pub struct Text {
    pub(crate) content: String,
    pub(crate) font: Option<String>,
    pub(crate) color: Option<(f64, f64, f64, f64)>,
    pub(crate) alignment: Alignment,
}

impl Text {
    /// Create a new Text.
    pub fn new<T: Into<String>>(content: T) -> Self {
        Text {
            content: content.into(),
            font: None,
            color: None,
            alignment: Alignment::CENTER,
        }
    }

    /// Set the font of the text.
    pub fn font<T: Into<String>>(mut self, font: T) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Set the foreground color of the text.
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
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
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
///                     .alignment(Alignmet::CENTER)
///                     .min_width(500);
/// ```
///
/// [`min_width`]: #method.min_width
pub struct Background {
    pub(crate) color: Option<u32>,
    pub(crate) image: Option<DynamicImage>,
    pub(crate) alignment: Alignment,
    pub(crate) min_width: u16,
}

impl Background {
    /// Create a background from an image.
    pub fn new_image(image: DynamicImage) -> Self {
        Background {
            image: Some(image),
            color: None,
            alignment: Alignment::CENTER,
            min_width: 0,
        }
    }

    /// Create a background from a color.
    pub fn new_color(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Background {
            image: None,
            color: Some(util::color(red, green, blue, alpha)),
            alignment: Alignment::CENTER,
            min_width: 0,
        }
    }

    /// Set the alignment of the background image.
    ///
    /// This does nothing for a [`new_color`](#method.new_color) background.
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set the minimum width of the component.
    pub fn min_width(mut self, min_width: u16) -> Self {
        self.min_width = min_width;
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
/// use leechbar::{BarBuilder, Component, Text, Background, ComponentPosition, Alignment};
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
///     // First element on the left side
///     fn position(&mut self) -> ComponentPosition {
///         ComponentPosition::new(Alignment::CENTER, 0)
///     }
///
///     // Do this only once
///     fn timeout(&mut self) -> Option<Duration> {
///         None
///     }
///
///     // Ignore all events
///     fn event(&mut self) {}
/// }
///
/// // Create a new bar
/// let mut bar = BarBuilder::new().spawn().unwrap();
/// // Add an instance of your component to your bar
/// bar.add(MyComponent{});
/// // Start the event loop that handles all X events
/// bar.start_event_loop();
/// ```
///
/// [`Bar::add`]: struct.Bar.html#method.add
pub trait Component {
    /// The background of the component.
    /// Use `None` for no background.
    fn background(&mut self) -> Option<Background>;
    /// The alignment and ordinal of the component.
    fn position(&mut self) -> ComponentPosition;
    /// The text of the component.
    /// Use `None` for no text.
    fn text(&mut self) -> Option<Text>;
    /// The polling rate for this component. This is the time between redrawing the component.
    /// Use `None` for drawing this component once.
    fn timeout(&mut self) -> Option<Duration>;
    /// X.Org events. This is not implemented yet.
    fn event(&mut self); // TODO: Create event type
}
