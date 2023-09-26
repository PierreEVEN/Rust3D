use std::sync::Arc;
use core::engine::{App, Builder, Camera, Engine};
use core::renderer::Renderer;
use core::world::World;
use ecs::entity::GameObject;
use ecs::query::Query;
use gfx::mesh::Mesh;
use gfx::shader_instance::ShaderInstance;
use gfx::types::BackgroundColor;
use maths::vec4::{Vec4F32};
use plateform::window::{PlatformEvent, WindowCreateInfos};

mod gfx_demo;

#[derive(Default)]
pub struct TestApp {
    main_camera: GameObject,
    world: Arc<World>,
}

#[derive(Default)]
struct MeshComponent {
    mesh: Option<Arc<Mesh>>,
    material: Option<Arc<dyn ShaderInstance>>,
    val: u32,
    name: String
}

impl App for TestApp {
    fn pre_initialize(&mut self, _: &mut Builder) {}
    fn initialized(&mut self) {
        // Create world

        self.world = Engine::get().new_world();
        self.main_camera = self.world.add_object::<Camera>(Camera {});
        self.world.add_object::<MeshComponent>(MeshComponent { mesh: None, material: None, val: 5, name: "coucou je suis un mesh".to_string() });
        
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
        renderer.bind_window_surface(&main_window, &BackgroundColor::Color(Vec4F32::new(0.2, 0.2, 0.0, 1.0)));
        renderer.set_default_view(&self.main_camera);
        
        match renderer.present_node().find_node("g_buffers") {
            None => {}
            Some(g_buffer) => {
                g_buffer.add_render_function(|ecs| {
                    Query::<&mut MeshComponent>::new(ecs).for_each(|a| {
                        logger::warning!("render {} : {}", a.name, a.val)
                    });
                })
            }
        }

        Engine::get().add_renderer(renderer);        
    }

    fn new_frame(&mut self, _delta_seconds: f64) {}
    fn request_shutdown(&self) {}
    fn stopped(&self) {}
}


use std::alloc::{GlobalAlloc, System, Layout};

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

fn main() {
    let mut engine = Engine::new::<TestApp>();
    engine.start();
}