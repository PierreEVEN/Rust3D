use std::alloc::Layout;
use std::any::TypeId;

pub type ComponentID = u32;

pub trait Component {
    fn component_id() -> ComponentID;
}

/*
STRUCTURE
 */

pub struct ComponentData {
    id : TypeId,
}

/*
REGISTRY
 */

#[derive(Default)]
pub struct ComponentRegistry {
}

impl ComponentRegistry {
    pub fn get_layout(&self, id: ComponentID) -> Layout {
        todo!()
    }
}
