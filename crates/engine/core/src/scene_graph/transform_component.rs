use std::sync::{Arc, RwLock};

use gfx_maths::{Mat4, Quaternion, Vec3};
use legion::{Entity, World, WorldOptions};
use legion::storage::IntoComponentSource;

use crate::scene_graph::component_base::ComponentBase;

pub struct TransformComponent {
    // local
    position: Vec3,
    rotation: Quaternion,
    scale: Vec3,
    
    // world
    world_transform: Mat4,
    dirty: bool,
    
    // inheritance
    parent: Option<Entity>,
    children: Vec<Entity>,
    
    // context
    world: Arc<RwLock<World>>,
}

impl TransformComponent {
    pub fn new() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: Default::default(),
            world_transform: Default::default(),
            parent: None,
            children: vec![],
            world: Arc::new(Default::default()),
            dirty: true,
        }
    }
    
    // Get local
    pub fn local_position(&self) -> Vec3 { self.position }
    pub fn local_scale(&self) -> Vec3 { self.scale }
    pub fn local_rotation(&self) -> Quaternion { self.rotation }
    
    // Set local
    pub fn set_local_position(&mut self, position: Vec3) {
        self.position = position;
        self.mark_self_dirty();
    }
    pub fn set_local_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.mark_self_dirty();
    }
    pub fn set_local_rotation(&mut self, rotation: Quaternion) {
        self.rotation = rotation;
        self.mark_self_dirty();
    }
    
    // Get world
    pub fn world_position(&mut self) -> Vec3 {
        self.update_world_transforms();
        todo!()
    }
    pub fn world_rotation(&mut self) -> Quaternion {
        self.update_world_transforms();
        todo!()
    }
    pub fn world_scale(&mut self) -> Vec3 {
        self.update_world_transforms();
        todo!()
    }
    
    // Set world
    pub fn set_world_position(&mut self, position: Vec3) {
        self.position = position;
        self.mark_self_dirty();
    }
    pub fn set_world_rotation(&mut self, rotation: Quaternion) {
        self.rotation = rotation;
        self.mark_self_dirty();
    }
    pub fn set_world_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.mark_self_dirty();
    }

    // World transform
    fn update_world_transforms(&mut self) {
        self.dirty = false;
        todo!()
    }

    fn mark_self_dirty(&mut self) {
        if self.dirty {
            return;
        }
        self.dirty = true;
        for child in &self.children {}
        todo!()
    }
    
    // Inheritance
    pub fn attach(&mut self, parent: Entity) {
        self.parent = Some(parent);
        if let Some(mut entry) = self.world.write().unwrap().entry(parent) {
            if let Ok(component) = entry.get_component_mut::<TransformComponent>() {
                component.children.push(todo!());
            }
        }
    }

    pub fn detach(&mut self) {
        if let Some(parent) = self.parent {
            if let Some(mut entry) = self.world.write().unwrap().entry(parent) {
                if let Ok(component) = entry.get_component_mut::<TransformComponent>() {
                    component.children.remove(todo!());
                }
            }
        }
        self.parent = None;
    }
}

fn test() {
    
    let mut world = World::default();


    let demo_comp = world.push((TransformComponent::new(),));
    
    
}