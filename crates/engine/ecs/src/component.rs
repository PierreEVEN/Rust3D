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
    pub id : TypeId,
}

/*
REGISTRY
 */

#[derive(Default)]
pub struct ComponentRegistry {
}

impl ComponentRegistry {
    pub fn get_layout(&self, _id: ComponentID) -> Layout {
        todo!()
    }
}
