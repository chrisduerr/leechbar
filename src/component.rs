use chan::{self, Receiver};
use foreground::Foreground;
use background::Background;
use alignment::Alignment;
use event::Event;
use width::Width;

/// Trait for creating custom components.
///
/// This trait is used for the [`Bar::add`] method. You can use it to implement custom components
/// that change at runtime.
///
/// # Examples
///
/// To create a component you only have to create a struct that implements the `Component` trait.
/// This is how you create an empty component:
///
/// ```rust
/// use leechbar::Component;
///
/// struct MyComponent;
/// impl Component for MyComponent {}
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::Component;
    ///
    /// struct MyComponent;
    /// impl Component for MyComponent {
    ///     // This would never draw anything
    ///     fn update(&mut self) -> bool {
    ///         false
    ///     }
    /// }
    /// ```
    fn update(&mut self) -> bool {
        true
    }

    /// This is called whenever an event occurs that is related to this component.
    ///
    /// The return value is used to check if the component is supposed to be redrawn after the
    /// event has been processed.
    ///
    /// **Default:** `false`, do nothing when an event is received.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::{Component, Event};
    ///
    /// struct MyComponent;
    /// impl Component for MyComponent {
    ///     fn event(&mut self, event: Event) -> bool {
    ///         if let Event::ClickEvent(_) = event {
    ///             println!("Someone clicked on this component!");
    ///         }
    ///         false
    ///     }
    /// }
    /// ```
    fn event(&mut self, _event: Event) -> bool {
        false
    }

    /// This method controls the redraw-rate of the component. Every time the `Receiver` receives
    /// any message, the component is redrawn. This method is called only once when the component
    /// is added to the bar, dropping the `Sender` will stop the component from being redrawn
    /// without removing the current state from the bar.
    ///
    /// **Default:** Sender dropped immediately, component is drawn only once.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate leechbar;
    /// extern crate chan;
    /// use leechbar::Component;
    /// use std::time::Duration;
    /// use std::thread;
    ///
    /// struct MyComponent;
    /// impl Component for MyComponent {
    ///     // Redraw this component every 5 seconds
    ///     fn redraw_timer(&mut self) -> chan::Receiver<()> {
    ///         let (tx, rx) = chan::sync(0);
    ///
    ///         thread::spawn(move || loop {
    ///             thread::sleep(Duration::from_secs(5));
    ///             tx.send(());
    ///         });
    ///
    ///         rx
    ///     }
    /// }
    /// # fn main() {}
    /// ```
    fn redraw_timer(&mut self) -> Receiver<()> {
        let (_tx, rx) = chan::sync(0);
        rx
    }

    /// The background of the component.
    /// Use `None` for no background.
    ///
    /// **Default:** No background.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::{Component, Color, Background};
    ///
    /// struct MyComponent;
    /// impl Component for MyComponent {
    ///     // Fixed pink background color
    ///     fn background(&self) -> Background {
    ///         Color::new(255, 0, 255, 255).into()
    ///     }
    /// }
    /// ```
    fn background(&self) -> Background {
        Background::new()
    }

    /// The text of the component.
    ///
    /// **Default:** No foreground.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::{Bar, Component, Foreground, Text};
    ///
    /// struct MyComponent {
    ///     bar: Bar,
    /// }
    ///
    /// impl Component for MyComponent {
    ///     // Fixed "Hello, World!" text
    ///     fn foreground(&self) -> Foreground {
    ///         Text::new(&self.bar, "Hello, Word!", None, None).unwrap().into()
    ///     }
    /// }
    /// ```
    fn foreground(&self) -> Foreground {
        Foreground::new()
    }

    /// The alignment of the component.
    ///
    /// **Default:** [`Alignment::CENTER`](enum.Alignment.html#variant.CENTER)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::{Component, Alignment};
    ///
    /// struct MyComponent;
    /// impl Component for MyComponent {
    ///     // Put the component at the right of the bar
    ///     fn alignment(&self) -> Alignment {
    ///         Alignment::RIGHT
    ///     }
    /// }
    fn alignment(&self) -> Alignment {
        Alignment::CENTER
    }

    /// The width of the component.
    ///
    /// **Default:** No width restrictions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::{Component, Width};
    ///
    /// struct MyComponent;
    /// impl Component for MyComponent {
    ///     // Fixed 300 pixel width
    ///     fn width(&self) -> Width {
    ///         Width::new().fixed(300)
    ///     }
    /// }
    /// ```
    fn width(&self) -> Width {
        Width::new()
    }
}
