use std::sync::{RwLock, Weak};

use ecs::entity::GameObject;
use gfx::surface::GfxSurface;
use plateform::window::Window;

use crate::engine::Engine;
use crate::renderer::Renderer;

pub struct WorldView {
    surfaces: RwLock<Vec<Box<dyn GfxSurface>>>,
    renderer: Renderer,
}

impl WorldView {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            surfaces: Default::default(),
            renderer,
        }
    }

    pub fn attach_to(&self, _camera: GameObject) {
        //todo!();
    }

    pub fn add_window(&self, window: &Weak<dyn Window>) {
        self.surfaces.write().unwrap().push(Engine::get_mut().new_surface(window));
    }
    pub fn new_frame(&self) {
        for _surface in &*self.surfaces.read().unwrap() {
            self.renderer.draw();
        }
    }
}
