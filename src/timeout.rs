use std::sync::mpsc::Receiver;
use std::time::Duration;

/// Timeout for locking a component thread.
///
/// This timeout is used to determine the timeout between redrawing a certain component. It is
/// possible to use a fixed duration or an event-based system.
pub struct Timeout<'a> {
    pub(crate) duration: Option<Duration>,
    pub(crate) receiver: Option<&'a Receiver<()>>,
}

impl<'a> Timeout<'a> {
    /// Create an event-based timeout.
    ///
    /// This takes a [`Receiver`](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html)
    /// which will stop the timeout as soon as any data is received.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::Timeout;
    /// use std::sync::mpsc;
    ///
    /// let (tx, rx) = mpsc::channel();
    /// let timeout = Timeout::new_event(&rx);
    /// ```
    pub fn new_event(receiver: &'a Receiver<()>) -> Self {
        Self {
            duration: None,
            receiver: Some(receiver),
        }
    }

    /// Create a time-based timeout.
    ///
    /// This timeout will be stopped after a fixed
    /// [`Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leechbar::Timeout;
    /// use std::time::Duration;
    ///
    /// let timeout = Timeout::new_duration(Duration::from_secs(3));
    /// ```
    pub fn new_duration(duration: Duration) -> Self {
        Self {
            receiver: None,
            duration: Some(duration),
        }
    }
}
