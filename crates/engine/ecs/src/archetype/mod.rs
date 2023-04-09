pub mod signature;

use std::collections::HashMap;
use std::slice;
use crate::archetype::signature::ArchetypeSignature;

use crate::component::{ComponentID, ComponentRegistry};
use crate::entity::EntityID;

pub type ArchetypeID = u32;

pub struct ComponentData {
    data: Vec<u8>,
    id: ComponentID,
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
            let dst = self.data.as_mut_ptr().add(entity_index * self.type_size);
            let src = self.data.as_ptr().add(entity_index * self.type_size);
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
            let dst = self.data.as_mut_ptr().add(entity_index * self.type_size);
            let src = data.as_ptr();
            std::ptr::copy(src, dst, self.type_size)
        }
    }

    #[inline]
    pub fn as_component<'ecs, C>(&self) -> &'ecs [C] {
        unsafe {
            slice::from_raw_parts(self.data.as_ptr() as *const C, self.entity_count)
        }
    }

    #[inline]
    pub fn as_component_mut<'ecs, C>(&mut self) -> &'ecs mut [C] {
        unsafe {
            slice::from_raw_parts_mut(self.data.as_ptr() as *mut C, self.entity_count)
        }
    }

    pub fn raw_len(&self) -> usize {
        self.data.len()
    }

    pub fn item_size(&self) -> usize {
        self.type_size
    }

    pub fn bound_entities(&self) -> usize {
        self.entity_count
    }

    pub fn id(&self) -> &ComponentID {
        &self.id
    }
}

/*
STRUCTURE
 */
#[derive(Default)]
pub struct Archetype {
    pub data: Vec<ComponentData>,
    entities: Vec<EntityID>,
    signature: ArchetypeSignature,
}

impl Archetype {
    pub fn new(identifier: ArchetypeSignature, registry: &ComponentRegistry) -> Archetype {
        let mut data = vec![];

        for comp in identifier.ids() {
            data.push(ComponentData::new(*comp, registry.get_layout(comp).size()))
        }

        Archetype {
            data,
            entities: vec![],
            signature: identifier,
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

    pub fn update_component_data(&mut self, entity_index: &usize, id: &ComponentID, data: &[u8]) {
        for comp in &mut self.data {
            if comp.id == *id {
                comp.update_component_data(entity_index, data);
                break;
            }
        }
    }

    pub fn signature(&self) -> &ArchetypeSignature {
        &self.signature
    }

    pub fn entity_data(&self, entity_index: &usize) -> Vec<(ComponentID, Vec<u8>)> {
        let mut data = Vec::with_capacity(self.signature.count());
        for comp in &self.data {
            let start = entity_index * comp.type_size;
            let end = (entity_index + 1) * comp.type_size;

            let sub_vec = &comp.data[start..end];
            data.push((comp.id, sub_vec.into()))
        }
        data
    }

    pub fn component_index(&self, component: &ComponentID) -> usize {
        for (i, id) in self.signature.ids().iter().enumerate() {
            if id == component {
                return i;
            }
        }
        panic!("archetype doesn't contains given component");
    }

    pub fn entity_at(&self, entity_index: &usize) -> &EntityID {
        &self.entities[*entity_index]
    }

    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

/*
REGISTRY
 */

#[derive(Default)]
pub struct ArchetypeRegistry {
    pub archetypes: Vec<Archetype>,
    registry_map: HashMap<ArchetypeSignature, ArchetypeID>,
}

impl ArchetypeRegistry {
    pub fn find_or_create(&mut self, identifier: ArchetypeSignature, registry: &ComponentRegistry) -> ArchetypeID {
        match self.registry_map.get(&identifier) {
            None => {
                self.archetypes.push(Archetype::new(identifier.clone(), registry));
                self.registry_map.insert(identifier, (self.archetypes.len() - 1)  as ArchetypeID);
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

    #[inline]
    pub fn get_archetype_mut(&mut self, id: &ArchetypeID) -> &mut Archetype {
        self.archetypes.get_mut(*id as usize).unwrap_or_else(|| panic!("Requested archetype id '{id}' is not valid"))
    }

    pub fn archetype_count(&self) -> usize { self.archetypes.len() }

    pub fn match_archetypes(&self, id: &ArchetypeSignature) -> Vec<ArchetypeID> {

        let mut ids = vec![];

        for (i, archetype) in self.archetypes.iter().enumerate() {
            if archetype.signature() & id {
                ids.push(i as ArchetypeID)
            }
        }

        ids
    }
}
