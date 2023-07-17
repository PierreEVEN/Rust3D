use std::alloc::Layout;
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::mem::{align_of, size_of};

pub type ComponentID = TypeId;

/*
TRAIT
 */

pub trait Component {
    
}

/*
STRUCTURE
 */

pub struct ComponentData {
    pub layout: Layout,
    pub name: &'static str,
}

impl ComponentData {
    pub fn new<C: Sized + Any>() -> ComponentData {
        Self {
            layout: Layout::from_size_align(size_of::<C>(), align_of::<C>()).expect("layout error"),
            name: type_name::<C>(),
        }
    }
}

/*
REGISTRY
 */

#[derive(Default)]
pub struct ComponentRegistry {
    components: HashMap<ComponentID, ComponentData>,
}

impl ComponentRegistry {
    pub fn contains<C: Any>(&self) -> bool {
        self.components.contains_key(&ComponentID::of::<C>())
    }

    pub fn register_component<C: Any>(&mut self) {
        self.components
            .insert(ComponentID::of::<C>(), ComponentData::new::<C>());
    }

    pub fn get_layout(&self, id: &ComponentID) -> &Layout {
        &self
            .components
            .get(id)
            .expect("component is not registered yet")
            .layout
    }

    pub fn component_infos(&self, id: &ComponentID) -> &ComponentData {
        self.components
            .get(id)
            .expect("component is not registered yet")
    }

    pub fn count(&self) -> usize {
        self.components.len()
    }

    pub fn ids(&self) -> Vec<ComponentID> {
        let mut keys = Vec::with_capacity(self.components.len());
        for k in self.components.keys() {
            keys.push(*k);
        }
        keys
    }
}
