use std::sync::{RwLock, Weak};
use ecs::entity::GameObject;
use gfx::surface::GfxSurface;
use logger::{debug_lvl};
use plateform::window::Window;
use crate::engine::Engine;

#[derive(Default)]
pub struct WorldView {
    surfaces: RwLock<Vec<Box<dyn GfxSurface>>>    
}

impl WorldView {    
    pub fn attach_to(&self, _camera: GameObject) {
        //todo!();
    }
    
    pub fn add_window(&self, window: &Weak<dyn Window>) {
        self.surfaces.write().unwrap().push(Engine::get_mut().new_surface(window));
    }
    pub fn new_frame(&self) {
        for _surface in &*self.surfaces.read().unwrap() {
            debug_lvl!(10, "Todo : draw surface here");
        }
    }
}
