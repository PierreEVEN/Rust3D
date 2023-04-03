use std::ptr::NonNull;
use crate::archetype::ArchetypeRegistry;
use crate::component::{Component, ComponentID, ComponentRegistry};
use crate::entity::{Entity, EntityRegistry};

#[derive(Default)]
pub struct Ecs {
    entities: EntityRegistry,
    components: ComponentRegistry,
    archetypes: ArchetypeRegistry,
}

impl Ecs {
    pub fn new(&mut self) -> Entity {
        self.entities.new()
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.entities.destroy(entity)
    }

    pub fn add<C: Component>(&mut self, entity: Entity, mut component: C) {
        unsafe { self.add_component(entity, C::component_id(), NonNull::new_unchecked(&mut component as *mut C as *mut u8)) }
    }

    pub fn remove<C: Component>(&mut self, entity: Entity) {
        self.remove_component(entity, C::component_id())
    }

    fn add_component(&mut self, entity: Entity, component: ComponentID, data: NonNull<u8>) {
        let current_arch_id = self.entities.get_archetype(entity);
        let mut current_arch = self.archetypes.get_archetype_mut(current_arch_id);

        // Already contains component
        if current_arch.contains(component) {
            current_arch.update_component(entity, component, data);
            return;
        }
    }
    fn remove_component(&mut self, entity: Entity, component: ComponentID) {
        todo!()
    }
}
