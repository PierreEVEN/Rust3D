use ecs::ecs::Ecs;
use ecs::entity::GameObject;
use logger::fatal;
use std::any::Any;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct World {
    ecs: Arc<RwLock<Ecs>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            ecs: Default::default(),
        }
    }

    pub fn add_object<ComponentT: Any>(&self, component: ComponentT) -> GameObject {
        match self.ecs.write() {
            Ok(mut ecs) => {
                let entity = ecs.create();
                ecs.add::<ComponentT>(entity, component);
                GameObject::new(entity, Arc::downgrade(&self.ecs))
            }
            Err(_) => {
                fatal!("failed to create object")
            }
        }
    }
}
