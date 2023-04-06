use std::any::{Any, type_name};
use std::collections::HashMap;
use std::mem::size_of;

use crate::archetype::{ArchetypeID, ArchetypeRegistry};
use crate::component::{ComponentID, ComponentRegistry};
use crate::entity::EntityID;
use crate::id_generator::IdGenerator;
use crate::system::SystemID;

#[derive(Default)]
pub struct Ecs {
    entity_registry: HashMap<EntityID, (ArchetypeID, usize)>,
    entity_id_manager: IdGenerator<EntityID>,
    components: ComponentRegistry,
    archetypes: ArchetypeRegistry,
}

impl Ecs {
    pub fn create(&mut self) -> EntityID {
        let new_id = self.entity_id_manager.acquire();
        self.entity_registry.insert(new_id, (EntityID::MAX, usize::MAX));
        new_id
    }

    pub fn destroy(&mut self, entity: EntityID) {
        match self.entity_registry.remove(&entity) {
            None => {}
            Some((entity_archetype, entity_index)) => {
                if entity_archetype != ArchetypeID::MAX {
                    self.archetypes.get_archetype_mut(&entity_archetype).drop_entity(&entity_index);
                }
            }
        };
        self.entity_id_manager.release(&entity);
    }


    pub fn add<C: Any>(&mut self, entity: EntityID, mut component: C) {
        let data = unsafe {
            Vec::from_raw_parts(
                &mut component as *mut C as *mut u8,
                size_of::<C>(),
                size_of::<C>())
        };

        if !self.components.contains::<C>() { self.components.register_component::<C>(); }

        self.add_component(entity, ComponentID::of::<C>(), data.as_slice());
    }

    pub fn remove<C: Any>(&mut self, entity: EntityID) {
        self.remove_component::<C>(entity, ComponentID::of::<C>())
    }

    fn add_component(&mut self, entity: EntityID, component: ComponentID, component_data: &[u8]) {
        // Retrieve archetype and internal entity index
        let (old_archetype_id, old_entity_index) = *self.entity_registry.get_mut(&entity).expect("The given entity is not registered yet");

        let mut data = vec![];

        let new_components = if old_archetype_id == ArchetypeID::MAX {
            // No components are bound to input entity
            vec![component]
        } else {
            // Retrieve data from existing archetype, then drop entity from it
            let old_archetype = self.archetypes.get_archetype_mut(&old_archetype_id);

            let mut component_ids = old_archetype.components().clone();
            data = old_archetype.entity_data(&old_entity_index).clone();

            // Update swapped entity indexes
            let swapped_entity_index = old_archetype.last_index();
            let swapped_entity = old_archetype.entity_at(&swapped_entity_index).clone();
            self.entity_registry.insert(swapped_entity, (old_archetype_id, swapped_entity_index));

            // Remove entity data
            old_archetype.drop_entity(&old_entity_index);
            component_ids.push(component);
            component_ids

        };

        // Find an archetype containing desired components
        let new_archetype_id = self.archetypes.find_or_create(new_components.as_slice(), &self.components);
        let new_archetype = self.archetypes.get_archetype_mut(&new_archetype_id);

        // Register entity into new archetype
        let new_entity_index = new_archetype.push_entity(entity);

        // Copy component data
        for (comp_id, comp_data) in data {
            new_archetype.update_component_data(&new_entity_index, &comp_id, comp_data)
        }
        new_archetype.update_component_data(&new_entity_index, &component, component_data.into());

        // Update entity_registry infos
        self.entity_registry.insert(entity, (new_archetype_id, new_entity_index));
    }

    fn remove_component<C: Any>(&mut self, entity: EntityID, component: ComponentID) {
        // Retrieve archetype and internal entity index
        let (old_archetype_id, old_entity_index) = *self.entity_registry.get_mut(&entity).expect("The given entity is not registered yet");

        let mut _data = vec![];

        let new_components = if old_archetype_id == ArchetypeID::MAX {
            panic!("Current entity doesn't contains any components")
        } else {
            // Retrieve data from existing archetype, then drop entity from it
            let old_archetype = self.archetypes.get_archetype_mut(&old_archetype_id);

            let mut component_ids = old_archetype.components().clone();
            _data = old_archetype.entity_data(&old_entity_index).clone();
            
            // Update swapped entity indexes
            let swapped_entity_index = old_archetype.last_index();
            let swapped_entity = old_archetype.entity_at(&swapped_entity_index).clone();
            self.entity_registry.insert(swapped_entity, (old_archetype_id, swapped_entity_index));

            // Remove entity data
            old_archetype.drop_entity(&old_entity_index);
            let mut index = usize::MAX;
            for i in 0..component_ids.len() { //@TODO : improve component sorting
                if component_ids[i] == component {
                    index = i;
                }
            }
            assert_ne!(index, usize::MAX, "Entity '{entity}' does not contains component '{}'", type_name::<C>());
            component_ids.swap_remove(index);
            component_ids
        };

        // Empty
        if new_components.is_empty() {
            self.entity_registry.insert(entity, (ArchetypeID::MAX, usize::MAX));
            return;
        }

        // Find an archetype containing desired components
        let new_archetype_id = self.archetypes.find_or_create(new_components.as_slice(), &self.components);
        let new_archetype = self.archetypes.get_archetype_mut(&new_archetype_id);

        // Register entity into new archetype
        let new_entity_index = new_archetype.push_entity(entity);

        // Copy component data
        for (comp_id, comp_data) in _data {
            new_archetype.update_component_data(&new_entity_index, &comp_id, comp_data)
        }

        // Update entity_registry infos
        self.entity_registry.insert(entity, (new_archetype_id, new_entity_index));
    }
    
    pub fn add_system<C>(&mut self, _name: &str, _test: C) -> SystemID {
        SystemID {}
    }
}
