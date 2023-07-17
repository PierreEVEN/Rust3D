use std::any::Any;
use std::collections::HashMap;
use std::mem::size_of;
use std::{mem, slice};

use crate::archetype::signature::ArchetypeSignature;
use crate::archetype::{Archetype, ArchetypeID, ArchetypeRegistry};
use crate::component::{ComponentID, ComponentRegistry};
use crate::entity::EntityID;
use crate::id_generator::IdGenerator;

#[derive(Default)]
pub struct Ecs {
    entity_registry: HashMap<EntityID, (ArchetypeID, usize)>,
    entity_id_manager: IdGenerator<EntityID>,
    pub components: ComponentRegistry,
    archetypes: ArchetypeRegistry,
}

impl Ecs {
    pub fn create(&mut self) -> EntityID {
        let new_id = self.entity_id_manager.acquire();
        self.entity_registry
            .insert(new_id, (EntityID::MAX, usize::MAX));
        new_id
    }

    pub fn destroy(&mut self, entity: EntityID) {
        match self.entity_registry.remove(&entity) {
            None => {}
            Some((entity_archetype, entity_index)) => {
                if entity_archetype != ArchetypeID::MAX {
                    // Update swapped entity indexes
                    let archetype = self.archetypes.get_archetype(&entity_archetype);
                    if archetype.entity_count() > 0 {
                        let swapped_entity_index = archetype.entity_count() - 1;
                        let swapped_entity = *archetype.entity_at(&swapped_entity_index);
                        self.entity_registry
                            .insert(swapped_entity, (entity_archetype, entity_index));
                    }
                    self.archetypes
                        .get_archetype_mut(&entity_archetype)
                        .drop_entity(&entity_index);
                }
                self.entity_id_manager.release(&entity);
            }
        };
    }

    pub fn match_archetypes(&self, id: &ArchetypeSignature) -> Vec<ArchetypeID> {
        self.archetypes.match_archetypes(id)
    }

    pub fn get_archetype(&self, id: &ArchetypeID) -> &Archetype {
        self.archetypes.get_archetype(id)
    }

    #[inline]
    pub fn get_archetype_mut(&mut self, id: &ArchetypeID) -> &mut Archetype {
        self.archetypes.get_archetype_mut(id)
    }

    pub fn add<C: Any>(&mut self, entity: EntityID, component: C) {
        let data =
            unsafe { slice::from_raw_parts(&component as *const C as *const u8, size_of::<C>()) };
        mem::forget(component);
        
        if !self.components.contains::<C>() {
            self.components.register_component::<C>();
        }
        
        self.add_component(entity, ComponentID::of::<C>(), data);
    }

    pub fn remove<C: Any>(&mut self, entity: EntityID) {
        self.remove_component::<C>(entity, ComponentID::of::<C>())
    }

    fn add_component(&mut self, entity: EntityID, component: ComponentID, component_data: &[u8]) {
        // Retrieve archetype and internal entity index
        let (old_archetype_id, old_entity_index) = *self
            .entity_registry
            .get_mut(&entity)
            .expect("The given entity is not registered yet");

        let mut data = vec![];

        let new_components = if old_archetype_id == ArchetypeID::MAX {
            // No components are bound to input entity
            vec![component].into()
        } else {
            // Retrieve data from existing archetype, then drop entity from it
            let old_archetype = self.archetypes.get_archetype_mut(&old_archetype_id);

            data = old_archetype.entity_data(&old_entity_index);

            // Update swapped entity indexes
            if old_archetype.entity_count() > 0 {
                let swapped_entity_index = old_archetype.entity_count() - 1;
                let swapped_entity = *old_archetype.entity_at(&swapped_entity_index);
                self.entity_registry
                    .insert(swapped_entity, (old_archetype_id, old_entity_index));
            }

            // Remove entity data
            old_archetype.drop_entity(&old_entity_index);
            old_archetype.signature().insert(component)
        };

        // Find an archetype containing desired components
        let new_archetype_id = self
            .archetypes
            .find_or_create(new_components, &self.components);
        let new_archetype = self.archetypes.get_archetype_mut(&new_archetype_id);

        // Register entity into new archetype
        let new_entity_index = new_archetype.push_entity(entity);

        // Copy component data
        for (comp_id, comp_data) in data {
            new_archetype.update_component_data(&new_entity_index, &comp_id, comp_data.as_slice())
        }
        
        new_archetype.update_component_data(&new_entity_index, &component, component_data);

        // Update entity_registry infos
        self.entity_registry
            .insert(entity, (new_archetype_id, new_entity_index));
    }

    fn remove_component<C: Any>(&mut self, entity: EntityID, component: ComponentID) {
        // Retrieve archetype and internal entity index
        let (old_archetype_id, old_entity_index) = *self
            .entity_registry
            .get_mut(&entity)
            .expect("The given entity is not registered yet");

        let mut _data = vec![];

        let new_components = if old_archetype_id == ArchetypeID::MAX {
            logger::fatal!("Current entity doesn't contains any components")
        } else {
            // Retrieve data from existing archetype, then drop entity from it
            let old_archetype = self.archetypes.get_archetype_mut(&old_archetype_id);

            _data = old_archetype.entity_data(&old_entity_index);

            // Update swapped entity indexes
            if old_archetype.entity_count() > 0 {
                let swapped_entity_index = old_archetype.entity_count() - 1;
                let swapped_entity = *old_archetype.entity_at(&swapped_entity_index);
                self.entity_registry
                    .insert(swapped_entity, (old_archetype_id, old_entity_index));
            }
            // Remove entity data
            old_archetype.signature().erase(component)
        };

        // Empty
        if new_components.is_empty() {
            self.entity_registry
                .insert(entity, (ArchetypeID::MAX, usize::MAX));
            return;
        }

        // Find an archetype containing desired components
        let new_archetype_id = self
            .archetypes
            .find_or_create(new_components, &self.components);
        let new_archetype = self.archetypes.get_archetype_mut(&new_archetype_id);

        // Register entity into new archetype
        let new_entity_index = new_archetype.push_entity(entity);

        // Copy component data
        for (comp_id, comp_data) in _data {
            new_archetype.update_component_data(&new_entity_index, &comp_id, comp_data.as_slice())
        }

        // Update entity_registry infos
        self.entity_registry
            .insert(entity, (new_archetype_id, new_entity_index));
    }

    pub fn print_stats(&self) {
        logger::debug!(
            "--Entities : [{}] -> [allocated={}:free={}]",
            self.entity_registry.len(),
            self.entity_id_manager.allocated_name(),
            self.entity_id_manager.pool_size()
        );
        logger::debug!("--Components : [count={}] ", self.components.count());
        for comp in &self.components.ids() {
            let comp = self.components.component_infos(comp);
            logger::debug!(
                "    {} [{}::{}]",
                comp.name,
                comp.layout.size(),
                comp.layout.align()
            );
        }
        logger::debug!(
            "--Archetypes : [count={}] ",
            self.archetypes.archetypes.len()
        );
        let mut i = 0;
        for archetype in &self.archetypes.archetypes {
            let mut signature = String::new();
            let mut first = true;
            for comp in archetype.signature().ids() {
                if first {
                    first = false
                } else {
                    signature.push_str("++");
                }
                signature.push_str(self.components.component_infos(comp).name);
            }
            logger::debug!(
                "    ({signature}) -> [count={}]",
                archetype.entity_count()
            );

            for data in &archetype.data {
                logger::debug!(
                    "      {} -> [entities={}:item_size={}]",
                    self.components.component_infos(data.id()).name,
                    data.bound_entities(),
                    data.item_size()
                );
                assert_eq!(data.bound_entities() * data.item_size(), data.raw_len());
            }

            i += 1;
            if i > 100 {
                logger::debug!(
                    "     +{} more",
                    self.archetypes.archetypes.len() as i64 - i + 1
                );
                break;
            }
        }
    }
}
