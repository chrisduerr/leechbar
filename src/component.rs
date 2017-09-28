use std::time::Duration;
use image::DynamicImage;

// Alignment and ordinal of a component
pub enum ComponentPosition {
    LEFT(u32),
    CENTER(u32),
    RIGHT(u32),
}

impl ComponentPosition {
    // Use the position and alignment of the item to get a unique id
    pub fn unique_id(&self) -> u32 {
        match *self {
            ComponentPosition::LEFT(ordinal) => ordinal * 3,
            ComponentPosition::CENTER(ordinal) => ordinal * 3 + 1,
            ComponentPosition::RIGHT(ordinal) => ordinal * 3 + 2,
        }
    }
}

// Trait for components
pub trait Component {
    fn background(&mut self) -> Option<DynamicImage>;
    fn position(&mut self) -> ComponentPosition;
    fn text(&mut self) -> Option<String>;
    fn timeout(&mut self) -> Duration;
    fn event(&mut self); // TODO: Create event type
}
