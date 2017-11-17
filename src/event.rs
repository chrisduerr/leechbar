use xcb::{ButtonPressEvent, MotionNotifyEvent};
use util::geometry::Geometry;

/// Event that indicates interaction with the component.
pub enum Event {
    /// User clicked on the component.
    ClickEvent(ClickEvent),
    /// User moved the mouse inside of the component.
    MotionEvent(MotionEvent),
}

impl<'a> From<&'a ButtonPressEvent> for Event {
    fn from(event: &'a ButtonPressEvent) -> Event {
        Event::ClickEvent(ClickEvent {
            button: MouseButton::new(event.detail()),
            position: Geometry::new(event.event_x(), event.event_y(), 0, 0),
        })
    }
}

impl<'a> From<&'a MotionNotifyEvent> for Event {
    fn from(event: &'a MotionNotifyEvent) -> Event {
        Event::MotionEvent(MotionEvent {
            position: Geometry::new(event.event_x(), event.event_y(), 0, 0),
        })
    }
}

/// Mouse Buttons.
///
/// This is used by the [`ClickEvent`](struct.ClickEvent.html) to indicate which mouse button has been
/// pressed.
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
}

impl MouseButton {
    fn new(index: u8) -> Self {
        match index {
            5 => MouseButton::WheelDown,
            4 => MouseButton::WheelUp,
            3 => MouseButton::Right,
            2 => MouseButton::Middle,
            _ => MouseButton::Left,
        }
    }
}

/// Mouse click on the component.
///
/// This event indicates that the user has clicked inside the component.
pub struct ClickEvent {
    /// The mouse button which has been used to click on the component.
    pub button: MouseButton,
    /// The position releative to the top-left of the component.
    pub position: Geometry,
}

/// Motion inside the component.
///
/// This event indicates that the user has moved the mouse inside the component.
pub struct MotionEvent {
    /// The position the user moved the mouse to.
    pub position: Geometry,
}
