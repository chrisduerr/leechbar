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
///     fn background(&mut self) -> Background {
///         Background::new()
///     }
///
///     // Do not print any text
///     fn foreground(&mut self) -> Option<Foreground> {
///         None
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
///     // Always redraw the component
///     fn redraw(&mut self) -> bool {
///         true
///     }
/// }
///
/// let component = MyComponent;
/// ```
///
/// [`Bar::add`]: struct.Bar.html#method.add
pub trait Component {
    /// The background of the component.
    /// Use `None` for no background.
    ///
    /// **Default:** No background.
    fn background(&mut self) -> Background {
        Background::new()
    }

    /// The text of the component.
    ///
    /// **Default:** `None`, no foreground.
    fn foreground(&mut self) -> Option<Foreground> {
        None
    }

    /// The alignment of the component.
    ///
    /// **Default:** [`Alignment::CENTER`](enum.Alignment.html#variant.CENTER)
    fn alignment(&mut self) -> Alignment {
        Alignment::CENTER
    }

    /// The width of the component.
    ///
    /// **Default:** No width restrictions.
    fn width(&mut self) -> Width {
        Width::new()
    }

    /// The polling rate for this component. This is the time between redrawing the component.
    /// Use `None` for drawing this component once.
    ///
    /// **Default:** `None`, draw component once.
    fn timeout(&mut self) -> Option<Duration> {
        None
    }

    /// Checked before redrawing a component.
    /// Returning `false` will not redraw the component and might save resources.
    ///
    /// This is called after [`foreground`](#method.foreground),
    /// [`background`](#method.background) and [`width`](#method.width).
    ///
    /// **Default:** Always redraw.
    fn redraw(&mut self) -> bool {
        true
    }
}
