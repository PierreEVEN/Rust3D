
pub type ComponentID = u32;

pub trait Component {
    fn component_id() -> ComponentID;
}

#[derive(Default)]
pub struct ComponentRegistry {}

impl ComponentRegistry {}
