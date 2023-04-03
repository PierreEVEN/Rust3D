use std::ptr::NonNull;
use crate::component::ComponentID;
use crate::entity::Entity;

pub type ArchetypeID = u32;

#[derive(Default)]
pub struct Archetype {
    components: Vec<ComponentID>,
}

impl Archetype {
    pub fn contains(&self, component: ComponentID) -> bool {
        for comp in &self.components {
            if *comp == component {
                return true
            }
        }
        false
    }

    pub fn update_component(&mut self, entity: Entity, component: ComponentID, data: NonNull<u8>) {}
}

#[derive(Default)]
pub struct ArchetypeRegistry {
    archetypes: Vec<Archetype>
}

impl ArchetypeRegistry {
    pub fn get_archetype(&self, id: ArchetypeID) -> &Archetype {
        &self.archetypes[id as usize]
    }
    
    pub fn get_archetype_mut(&mut self, id: ArchetypeID) -> &mut Archetype {
        &mut self.archetypes[id as usize]
    }
}
