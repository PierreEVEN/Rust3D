use std::sync::Arc;

use core::engine::{App, Builder, Camera, Engine};
use core::renderer::Renderer;
use core::world::World;
use ecs::entity::GameObject;
use plateform::input_system::{InputAction, InputMapping, KeyboardKey};
use plateform::window::{PlatformEvent, WindowCreateInfos};

mod gfx_demo;

#[derive(Default)]
pub struct TestApp {}

impl App for TestApp {
    fn pre_initialize(&mut self, _: &mut Builder) {}
    fn initialized(&mut self) {
        // Create world
        self.primary_camera = self.world.add_object::<Camera>(Camera {});

        // Create main window
        let main_window = Engine::get().platform().create_window(WindowCreateInfos::default_named("Rust3D Editor")).unwrap();
        main_window.upgrade().unwrap().show();
        main_window.upgrade().unwrap().bind_event(
            PlatformEvent::WindowClosed,
            Box::new(|_| {
                Engine::get().shutdown();
            }),
        );

        // Create world view and set output to main window
        let world_view = Engine::get().create_view(Renderer::default_deferred());
        world_view.upgrade().unwrap().add_window(&main_window);
    }

    fn new_frame(&mut self, _delta_seconds: f64) {}
    fn request_shutdown(&self) {}
    fn stopped(&self) {}
}

fn main() {
    let mut engine = Engine::new::<TestApp>();
    engine.start();
}