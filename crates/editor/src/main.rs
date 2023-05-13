use core::engine::{App, Builder, Camera, Engine};
use core::renderer::Renderer;
use core::world::World;
use ecs::entity::GameObject;
use plateform::window::{PlatformEvent, WindowCreateInfos};

mod gfx_demo;

#[derive(Default)]
pub struct TestApp {
    main_camera: GameObject,
    world: World,
}

impl App for TestApp {
    fn pre_initialize(&mut self, _: &mut Builder) {}
    fn initialized(&mut self) {
        // Create world
        self.main_camera = self.world.add_object::<Camera>(Camera {});

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
        let renderer = Renderer::default_deferred();
        renderer.add_window(&main_window);
        renderer.attach_to(&self.main_camera);
        Engine::get().add_renderer(renderer);
    }

    fn new_frame(&mut self, _delta_seconds: f64) {}
    fn request_shutdown(&self) {}
    fn stopped(&self) {}
}

fn main() {
    let mut engine = Engine::new::<TestApp>();
    engine.start();
}