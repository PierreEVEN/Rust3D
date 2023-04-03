use std::collections::LinkedList;
use std::ptr::NonNull;
use crate::archetype::ArchetypeID;

pub type EntityID = u32;

#[derive(Default)]
pub struct EntityMetaData {
    pub archetype_id: ArchetypeID,
}

#[derive(Default)]
pub struct EntityRegistry {
    data: Vec<EntityMetaData>,
    entities: Vec<EntityID>,
    free_ids: LinkedList<EntityID>,
    max_id: EntityID,
}

impl EntityRegistry {
    fn gen_id(&mut self) -> EntityID {
        if !self.free_ids.is_empty() {
            return self.free_ids.pop_back().expect("list should not be empty there");
        }
        self.max_id += 1;
        return self.max_id - 1;
    }

    fn release_id(&mut self, id: EntityID) {
        self.free_ids.push_back(id);
    }

    pub fn new(&mut self) -> Entity {
        Entity::new(self.gen_id())
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.release_id(entity.id())
    }
    
    pub fn get_archetype(&self, entity: Entity) -> ArchetypeID {
        self.data[entity.id() as usize].archetype_id
    }
}

#[derive(Default, Copy, Clone)]
pub struct Entity {
    in_id: EntityID,
}

impl Entity {
    pub fn new(id: EntityID) -> Entity {
        Entity { in_id: id }
    }
    pub fn id(&self) -> EntityID { self.in_id }
}
