use std::collections::HashMap;
use std::mem::size_of;
use crate::archetype::{ArchetypeID, ArchetypeRegistry};
use crate::component::{Component, ComponentID, ComponentRegistry};
use crate::entity::{EntityID};

#[derive(Default)]
pub struct Ecs {
    entity_registry: HashMap<EntityID, (ArchetypeID, usize)>,
    components: ComponentRegistry,
    archetypes: ArchetypeRegistry,
}

impl Ecs {
    pub fn add<C: Component>(&mut self, entity: EntityID, mut component: C) {
        let data = unsafe {
            Vec::from_raw_parts(
                &mut component as *mut C as *mut u8,
                size_of::<C>(),
                size_of::<C>())
        };

        self.add_component(entity, C::component_id(), data.as_slice());
    }

    pub fn remove<C: Component>(&mut self, entity: EntityID) {
        self.remove_component(entity, C::component_id())
    }

    fn add_component(&mut self, entity: EntityID, component: ComponentID, component_data: &[u8]) {
        let (old_archetype_id, old_entity_index) = self.entity_registry.get_mut(&entity).expect("given entity is nor registered in archetypes");

        let old_components = self.archetypes.get_archetype(old_archetype_id).components().clone();
        
        let data = self.archetypes.get_archetype(old_archetype_id).entity_data(*old_entity_index).clone();

        self.archetypes.get_archetype_mut(old_archetype_id).drop_entity(*old_entity_index);

        let mut new_components = old_components;
        new_components.push(component);

        let new_archetype = self.archetypes.find_or_create(new_components.as_slice(), &self.components);

        let new_index = self.archetypes.get_archetype_mut(&new_archetype).push_entity(entity);
        for (comp_id, comp_data) in data {
            self.archetypes.get_archetype_mut(&new_archetype).update_component_data(new_index, &comp_id, comp_data)
        }
        self.archetypes.get_archetype_mut(&new_archetype).update_component_data(new_index, &component, component_data.into());
        self.entity_registry.insert(entity, (new_archetype, new_index));
    }

    fn remove_component(&mut self, entity: EntityID, component: ComponentID) {
        let (old_archetype_id, old_entity_index) = self.entity_registry.get_mut(&entity).expect("given entity is nor registered in archetypes");

        let old_components = self.archetypes.get_archetype(old_archetype_id).components().clone();
        
        let data = self.archetypes.get_archetype(old_archetype_id).entity_data(*old_entity_index);

        self.archetypes.get_archetype_mut(old_archetype_id).drop_entity(*old_entity_index);

        let mut new_components = old_components;
        new_components.swap_remove(new_components.binary_search(&component).expect("component not found"));

        let new_archetype = self.archetypes.find_or_create(new_components.as_slice(), &self.components);

        let new_index = self.archetypes.get_archetype_mut(&new_archetype).push_entity(entity);
        for (comp_id, comp_data) in data {
            self.archetypes.get_archetype_mut(&new_archetype).update_component_data(new_index, &comp_id, comp_data)
        }
        self.entity_registry.insert(entity, (new_archetype, new_index));
    }
}
