/// Alignment inside a container.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

    // Calculate the next id for a component
    pub(crate) fn id(&self, component_ids: &mut [u32; 3]) -> u32 {
        let index = match *self {
            Alignment::LEFT => 0,
            Alignment::CENTER => 1,
            Alignment::RIGHT => 2,
        };

        let return_val = component_ids[index];
        component_ids[index] += 3;
        return_val
    }
}
