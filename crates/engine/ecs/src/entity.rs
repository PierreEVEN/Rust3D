use std::sync::{Arc, RwLock, Weak};

use crate::ecs::Ecs;

pub type EntityID = u32;

#[derive(Clone)]
pub struct GameObject {
    entity_id: Arc<RwLock<EntityID>>,
    ecs_ref: Option<Weak<RwLock<Ecs>>>,
}

impl GameObject {
    pub fn new(entity: EntityID, ecs: Weak<RwLock<Ecs>>) -> Self {
        Self {
            entity_id: Arc::new(RwLock::new(entity)),
            ecs_ref: Some(ecs),
        }
    }

    pub fn is_valid(&self) -> bool {
        if *self.entity_id.read().unwrap() == EntityID::MAX {
            return false;
        }
        match &self.ecs_ref {
            None => false,
            Some(ecs) => ecs.weak_count() > 0,
        }
    }

    pub fn destroy(&self) {
        assert!(self.is_valid());

        if let Ok(mut entity_id) = self.entity_id.write() {
            match &self.ecs_ref {
                None => {}
                Some(ecs) => unsafe {
                    if let Ok(mut ecs) = ecs.as_ptr().as_ref().unwrap().write() {
                        ecs.destroy(*self.entity_id.read().unwrap());
                        *entity_id = EntityID::MAX;
                    }
                },
            }
        }
    }
}

impl Default for GameObject {
    fn default() -> Self {
        Self {
            entity_id: Arc::new(RwLock::new(u32::MAX)),
            ecs_ref: None,
        }
    }
}
