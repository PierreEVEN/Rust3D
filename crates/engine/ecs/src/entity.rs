use std::collections::LinkedList;

type EntityID = u32;
type ComponentID = u32;
type ArchetypeID = u32;

pub struct EntityRegistry {
    entities: Vec<EntityID>,
    free_ids: LinkedList<EntityID>,
    max_id: EntityID,
}

impl EntityRegistry {
    fn gen_id(&mut self) -> EntityID {
        if !self.free_ids.empty() {
            return self.free_ids.pop();
        }
        self.max_id += 1;
        return self.max_id - 1;
    }
    
    fn release_id(&mut self, id: EntityID) {
        self.free_ids.append(id);
    }
}

pub struct Archetype {
    
}

pub struct Entity {
    id: EntityID
}