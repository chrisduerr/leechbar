use chan::{self, Receiver};
use foreground::Foreground;
use background::Background;
use alignment::Alignment;
use event::Event;
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
/// # extern crate leechbar;
/// extern crate chan;
///
/// use leechbar::{Component, Background, Foreground, Alignment, Width};
/// use std::time::Duration;
/// use std::thread;
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
///     fn foreground(&self) -> Foreground {
///         Foreground::new()
///     }
///
///     // Put this element at the center of the bar
///     fn alignment(&self) -> Alignment {
///         Alignment::CENTER
///     }
///
///     // Redraw every 5 seconds
///     fn redraw_timer(&mut self) -> chan::Receiver<()> {
///         let (tx, rx) = chan::sync(0);
///
///         // Start thread for sending update requests
///         // Then send the updates every 5 seconds
///         thread::spawn(move || loop {
///             thread::sleep(Duration::from_secs(5));
///             let _ = tx.send(());
///         });
///
///         rx
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
/// fn main() {
///     let component = MyComponent;
/// }
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

    /// This is called whenever an event occurs that is related to this component.
    ///
    /// The return value is used to check if the component is supposed to be redrawn after the
    /// event has been processed.
    ///
    /// **Default:** `false`, do nothing when an event is received.
    fn event(&mut self, _event: Event) -> bool {
        false
    }

    /// This method controls the redraw-rate of the component. Every time the `Receiver` receives
    /// any message, the component is redrawn. This method is called only once when the component
    /// is added to the bar, dropping the `Sender` will stop the component from being redrawn
    /// without removing the current state from the bar.
    ///
    /// **Default:** Sender dropped immediately, component is drawn only once.
    fn redraw_timer(&mut self) -> Receiver<()> {
        let (_tx, rx) = chan::sync(0);
        rx
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
    /// **Default:** No foreground.
    fn foreground(&self) -> Foreground {
        Foreground::new()
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
}
