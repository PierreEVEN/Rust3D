use std::collections::HashMap;
use std::ptr::NonNull;
use crate::archetype::{ArchetypeID, ArchetypeRegistry};
use crate::component::{Component, ComponentID, ComponentRegistry};
use crate::entity::{EntityID, EntityRegistry};

#[derive(Default)]
pub struct Ecs {
    entity_registry: HashMap<EntityID, ArchetypeID>,
    components: ComponentRegistry,
    archetypes: ArchetypeRegistry,
}

impl Ecs {
    pub fn add<C: Component>(&mut self, entity: EntityID, mut component: C) {
        unsafe { self.add_component(entity, C::component_id(), NonNull::new_unchecked(&mut component as *mut C as *mut u8)) }
    }

    pub fn remove<C: Component>(&mut self, entity: EntityID) {
        self.remove_component(entity, C::component_id())
    }

    fn add_component(&mut self, entity: EntityID, component: ComponentID, data: NonNull<u8>) {
        
        
        
        
        
        
        todo!()
    }
    fn remove_component(&mut self, entity: EntityID, component: ComponentID) {
        
        todo!()
    }
}
