use foreground::Foreground;
use background::Background;
use alignment::Alignment;
use std::time::Duration;
use width::Width;

/// Trait for creating custom components.
///
/// This trait is used for the [`Bar::add`] method. You can use it to implement custom components
/// that change at runtime. Each method takes `&mut self` and is called whenever the component
/// redraws, this allows mutating the struct of the component at runtime.
///
/// # Examples
///
/// ```rust
/// use leechbar::{Component, Background, Foreground, Alignment, Width};
/// use std::time::Duration;
///
/// struct MyComponent;
///
/// // You can define your own custom components like this
/// impl Component for MyComponent {
///     // No background image
///     fn background(&self) -> Background {
///         Background::new()
///     }
///
///     // Do not print any text
///     fn foreground(&self) -> Option<Foreground> {
///         None
///     }
///
///     // Put this element at the center of the bar
///     fn alignment(&self) -> Alignment {
///         Alignment::CENTER
///     }
///
///     // Do this only once
///     fn timeout(&self) -> Option<Duration> {
///         None
///     }
///
///     // No width restrictions
///     fn width(&self) -> Width {
///         Width::new()
///     }
///
///     // Always redraw component
///     fn update(&mut self) -> bool {
///         true
///     }
/// }
///
/// let component = MyComponent;
/// ```
///
/// [`Bar::add`]: struct.Bar.html#method.add
pub trait Component {
    /// This is the first thing called before redrawing a component.
    /// It can be used to modify the state of the struct implementing the `Component` trait.
    ///
    /// This method's return value determines if the component should be redrawn in this cycle,
    /// returning `false` instead of redrawing the same content will save resources.
    ///
    /// **Default:** `true`, component will always be redrawn.
    fn update(&mut self) -> bool {
        true
    }

    /// The background of the component.
    /// Use `None` for no background.
    ///
    /// **Default:** No background.
    fn background(&self) -> Background {
        Background::new()
    }

    /// The text of the component.
    ///
    /// **Default:** `None`, no foreground.
    fn foreground(&self) -> Option<Foreground> {
        None
    }

    /// The alignment of the component.
    ///
    /// **Default:** [`Alignment::CENTER`](enum.Alignment.html#variant.CENTER)
    fn alignment(&self) -> Alignment {
        Alignment::CENTER
    }

    /// The width of the component.
    ///
    /// **Default:** No width restrictions.
    fn width(&self) -> Width {
        Width::new()
    }

    /// The polling rate for this component. This is the time between redrawing the component.
    /// Use `None` for drawing this component once.
    ///
    /// **Default:** `None`, draw component once.
    fn timeout(&self) -> Option<Duration> {
        None
    }
}
