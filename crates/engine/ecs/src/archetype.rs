use std::alloc::Layout;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use crate::component::{ComponentID, ComponentRegistry};
use crate::entity::EntityID;

pub type ArchetypeID = u32;

#[derive(Default)]
pub struct ComponentData {
    id: ComponentID,
    data: Vec<u8>,
    type_size: usize,
    entity_count: usize,
}

impl ComponentData {
    pub fn new(id: ComponentID, type_size: usize) -> ComponentData {
        Self { id, data: vec![], type_size, entity_count: 0 }
    }

    pub fn extend(&mut self) {
        self.resize(self.entity_count + 1);
    }

    pub fn drop_index(&mut self, entity_index: usize) {
        unsafe {
            let dst = self.data.as_mut_ptr().offset((entity_index * self.type_size) as isize);
            let src = self.data.as_ptr().offset((entity_index * self.type_size) as isize);
            std::ptr::copy(src, dst, self.type_size);
        }

        self.resize(self.entity_count - 1);
        todo!("update swapped one index")
    }

    fn resize(&mut self, new_entity_count: usize) {
        self.entity_count = new_entity_count;
        self.data.resize(self.entity_count * self.type_size, 0);
    }

    pub fn get_component_data(&self, index: usize) -> &[u8] {
        &self.data.as_slice()[index * self.type_size..(index + 1) * self.type_size]
    }

    pub fn update_component_data(&mut self, entity_index: usize, data: &[u8]) {
        unsafe {
            let dst = self.data.as_mut_ptr().offset((entity_index * self.type_size) as isize);
            let src = data.as_ptr();
            std::ptr::copy(src, dst, self.type_size)
        }
    }
}

/*
STRUCTURE
 */

#[derive(Default)]
pub struct Archetype {
    data: Vec<ComponentData>,
    entities: Vec<EntityID>,
}

impl Archetype {
    pub fn new(components: &[ComponentID], registry: &ComponentRegistry) -> Archetype {
        let mut data = vec![];

        for comp in components {
            data.push(ComponentData::new(*comp, registry.get_layout(*comp).size()))
        }

        Archetype {
            data,
            entities: vec![],
        }
    }

    pub fn push_entity(&mut self, entity: EntityID) -> usize {
        self.entities.push(entity);
        for comp in &mut self.data {
            comp.extend();
        }
        self.entities.len() - 1
    }

    pub fn drop_entity(&mut self, entity_index: usize) {
        for comp in &mut self.data {
            comp.drop_index(entity_index);
        }
        self.entities.swap_remove(entity_index);
    }

    pub fn move_entity_to(&mut self, entity_index: usize, to: &mut Archetype) {
        let new_index = to.push_entity(self.entities[entity_index]);

        for comp_src in &self.data {
            for comp_dst in &mut to.data {
                if comp_src.id == comp_dst.id {
                    comp_dst.update_component_data(new_index, comp_src.get_component_data(entity_index));
                }
            }
        }

        self.drop_entity(entity_index);
    }
}

/*
REGISTRY
 */

#[derive(Default)]
pub struct ArchetypeRegistry {
    archetypes: Vec<Archetype>,
    registry_map: HashMap<Vec<ComponentID>, ArchetypeID>,
}

impl ArchetypeRegistry {
    pub fn find_or_get(&mut self, components: &[ComponentID], registry: &ComponentRegistry) -> ArchetypeID {
        match self.registry_map.get(components) {
            None => {
                self.archetypes.push(Archetype::new(components, registry));
                (self.archetypes.len() - 1) as ArchetypeID
            }
            Some(found_id) => {
                *found_id
            }
        }
    }

    pub fn get_archetype(&self, id: ArchetypeID) -> &Archetype {
        &self.archetypes[id as usize]
    }
    
    pub fn get_archetype_mut(&mut self, id: ArchetypeID) -> &mut Archetype {
        &mut self.archetypes[id as usize]
    }
}
