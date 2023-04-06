use std::alloc::Layout;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::mem::{align_of, size_of};

pub type ComponentID = TypeId;

/*
STRUCTURE
 */

pub struct ComponentData {
    pub layout: Layout,
}

impl ComponentData {
    pub fn new<C: Sized + Any>() -> ComponentData {
        Self {
            layout: Layout::from_size_align(size_of::<C>(), align_of::<C>()).expect("layout error")
        }
    }
}

/*
REGISTRY
 */

#[derive(Default)]
pub struct ComponentRegistry {
    components: HashMap<ComponentID, ComponentData>
}

impl ComponentRegistry {
    
    pub fn contains<C:Any>(&self) -> bool {
        self.components.contains_key(&ComponentID::of::<C>())
    }
    
    pub fn register_component<C:Any>(&mut self) {
        self.components.insert(ComponentID::of::<C>(), ComponentData::new::<C>());
    }
    
    pub fn get_layout(&self, id: &ComponentID) -> &Layout {
        &self.components.get(id).expect("component is not registered yet").layout
    }
}
