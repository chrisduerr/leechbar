use std::time::Duration;
use image::DynamicImage;

// Alignment of component or text
#[derive(Clone, Copy)]
pub enum Alignment {
    LEFT,
    CENTER,
    RIGHT,
}

// Alignment and position of a component
#[derive(Clone, Copy)]
pub struct ComponentPosition {
    alignment: Alignment,
    ordinal: u32,
}

impl ComponentPosition {
    // Create a new component position
    pub fn new(alignment: Alignment, ordinal: u32) -> Self {
        ComponentPosition { alignment, ordinal }
    }

    // Use the position and alignment of the item to get a unique id
    pub fn unique_id(&self) -> u32 {
        match self.alignment {
            Alignment::LEFT => self.ordinal * 3,
            Alignment::CENTER => self.ordinal * 3 + 1,
            Alignment::RIGHT => self.ordinal * 3 + 2,
        }
    }
}

// Struct for a text element
pub struct Text {
    pub content: String,
    pub font: Option<String>,
    pub alignment: Alignment,
}

impl Text {
    // Create a text element
    pub fn new<T: Into<String>>(content: T) -> Self {
        Text {
            content: content.into(),
            font: None,
            alignment: Alignment::LEFT,
        }
    }

    // Set the font of the text element
    pub fn font<T: Into<String>>(mut self, font: T) -> Self {
        self.font = Some(font.into());
        self
    }

    // Set the alignment of the text element
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

// Trait for components
pub trait Component {
    fn background(&mut self) -> Option<DynamicImage>;
    fn position(&mut self) -> ComponentPosition;
    fn text(&mut self) -> Option<Text>;
    fn timeout(&mut self) -> Duration;
    fn event(&mut self); // TODO: Create event type
}
