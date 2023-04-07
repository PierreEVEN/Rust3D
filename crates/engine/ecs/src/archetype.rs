use std::collections::HashMap;

use crate::component::{ComponentID, ComponentRegistry};
use crate::entity::EntityID;

pub type ArchetypeID = u32;

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

    pub fn drop_index(&mut self, entity_index: &usize) {
        unsafe {
            let dst = self.data.as_mut_ptr().offset((entity_index * self.type_size) as isize);
            let src = self.data.as_ptr().offset((entity_index * self.type_size) as isize);
            std::ptr::copy(src, dst, self.type_size);
        }

        self.resize(self.entity_count - 1);
    }

    fn resize(&mut self, new_entity_count: usize) {
        self.entity_count = new_entity_count;
        self.data.resize(self.entity_count * self.type_size, 0);
    }

    pub fn get_component_data(&self, index: &usize) -> &[u8] {
        &self.data.as_slice()[*index * self.type_size..(*index + 1) * self.type_size]
    }

    pub fn update_component_data(&mut self, entity_index: &usize, data: &[u8]) {
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
    components: Vec<ComponentID>,
}

impl Archetype {
    pub fn new(components: &[ComponentID], registry: &ComponentRegistry) -> Archetype {
        let mut data = vec![];

        for comp in components {
            data.push(ComponentData::new(*comp, registry.get_layout(comp).size()))
        }

        Archetype {
            data,
            entities: vec![],
            components: components.into(),
        }
    }

    pub fn push_entity(&mut self, entity: EntityID) -> usize {
        self.entities.push(entity);
        for comp in &mut self.data {
            comp.extend();
        }
        self.entities.len() - 1
    }

    pub fn drop_entity(&mut self, entity_index: &usize) {
        for comp in &mut self.data {
            comp.drop_index(entity_index);
        }
        self.entities.swap_remove(*entity_index);
    }

    pub fn update_component_data(&mut self, entity_index: &usize, id: &ComponentID, data: Vec<u8>) {
        for comp in &mut self.data {
            if comp.id == *id {
                comp.update_component_data(entity_index, data.as_slice());
                break;
            }
        }
    }
    
    pub fn components(&self) -> &Vec<ComponentID> {
        &self.components
    }

    pub fn entity_data(&self, entity_index: &usize) -> Vec<(ComponentID, Vec<u8>)> {
        let mut data = Vec::with_capacity(self.components.len());
        for comp in &self.data {
            let start = entity_index * comp.type_size;
            let end = (entity_index + 1) * comp.type_size;
            
            let sub_vec = &comp.data[start..end];
            data.push((comp.id, sub_vec.into()))
        }
        data
    }
    
    pub fn last_index(&self) -> usize {
        self.data[0].entity_count - 1
    }
    
    pub fn entity_at(&self, entity_index: &usize) -> &EntityID {
        &self.entities[*entity_index]
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
    pub fn find_or_create(&mut self, components: &[ComponentID], registry: &ComponentRegistry) -> ArchetypeID {
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

    pub fn get_archetype(&self, id: &ArchetypeID) -> &Archetype {
        &self.archetypes[*id as usize]
    }
    
    pub fn get_archetype_mut(&mut self, id: &ArchetypeID) -> &mut Archetype {
        self.archetypes.get_mut(*id as usize).expect(format!("Requested archetype id '{id}' is not valid").as_str())
    }
    
    pub fn archetype_count(&self) -> usize { self.archetypes.len() }
}
