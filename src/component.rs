use foreground::Foreground;
use background::Background;
use alignment::Alignment;
use std::time::Duration;
use width::Width;

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
    fn background(&mut self) -> Background;
    /// The alignment of the component.
    fn alignment(&mut self) -> Alignment;
    /// The text of the component.
    fn foreground(&mut self) -> Option<Foreground>;
    /// The width of the component.
    fn width(&mut self) -> Width;
    /// The polling rate for this component. This is the time between redrawing the component.
    /// Use `None` for drawing this component once.
    fn timeout(&mut self) -> Option<Duration>;
    /// X.Org events. This is not implemented yet.
    fn event(&mut self); // TODO: Create event type
}
