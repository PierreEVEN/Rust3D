use std::alloc::{GlobalAlloc, Layout, System};
use std::fs;
use std::mem::size_of;
use std::path::Path;
use std::slice::from_raw_parts;
use std::sync::Arc;

use core::base_assets::material_asset::MaterialAsset;
use core::engine::{App, Builder, Camera, Engine};
use core::renderer::Renderer;
use core::world::World;
use ecs::entity::GameObject;
use ecs::query::Query;
use gfx::buffer::BufferType;
use gfx::Gfx;
use gfx::mesh::{IndexBufferType, Mesh, MeshCreateInfos};
use gfx::shader::{ShaderProgramInfos, ShaderProgramStage};
use gfx::shader_instance::ShaderInstance;
use gfx::types::BackgroundColor;
use maths::vec3::Vec3F32;
use maths::vec4::Vec4F32;
use plateform::window::{PlatformEvent, WindowCreateInfos};

mod gfx_demo;


pub trait Material {
    fn instantiate(&self) -> Arc<dyn MaterialInstance>;
}

pub trait MaterialInstance {

}

#[derive(Default)]
struct MeshComponent {
    pub mesh: Option<Arc<Mesh>>,
    pub material: Option<Arc<dyn Material>>,
    pub name: String,
}

#[derive(Default)]
pub struct TestApp {
    main_camera: GameObject,
    world: Arc<World>,
}

impl App for TestApp {
    fn pre_initialize(&mut self, _: &mut Builder) {}
    fn initialized(&mut self) {
        // Create world

        self.world = Engine::get().new_world();
        self.main_camera = self.world.add_object::<Camera>(Camera {});

        // Create dummy mesh
        let vertices = vec![
            Vec3F32::new(0.0, 0.0, 0.0),
            Vec3F32::new(1.0, 0.0, 0.0),
            Vec3F32::new(1.0, 1.0, 0.0),
            Vec3F32::new(0.0, 1.0, 0.0)];
        let indices: Vec<u32> = vec![0, 2, 1, 2, 3, 1];
        let mesh = Gfx::get().create_mesh("un mesh".to_string(), &MeshCreateInfos {
            vertex_structure_size: size_of::<Vec3F32>() as u32,
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
            buffer_type: BufferType::Immutable,
            index_buffer_type: IndexBufferType::Uint16,
            vertex_data: Some(unsafe { Vec::from_raw_parts(vertices.as_ptr() as *mut Vec3F32 as *mut u8, vertices.len() * size_of::<Vec3F32>(), vertices.len() * size_of::<Vec3F32>()) }),
            index_data: Some(unsafe { Vec::from_raw_parts(indices.as_ptr() as *mut u32 as *mut u8, indices.len() * size_of::<Vec3F32>(), indices.len() * size_of::<u32>()) }),
        });

        // Create dummy material
        let material = MaterialAsset::new();
        material.set_shader_code(&Path::new("./data/shaders/basic.shb"), fs::read_to_string(Path::new("./data/shaders/basic.shb")).unwrap());
        
        self.world.add_object::<MeshComponent>(MeshComponent {
            mesh: Some(mesh),
            material: Some(material),
            name: "coucou je suis un mesh".to_string(),
        });

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
                g_buffer.add_render_function(|ecs, command_buffer| {
                    Query::<&mut MeshComponent>::new(ecs).for_each(|a| {
                        match &a.material {
                            None => {}
                            Some(material) => {
                                match &a.mesh {
                                    None => {}
                                    Some(mesh) => {
                                        command_buffer.bind_shader_instance(material);
                                        command_buffer.draw_mesh(mesh, 1, 0);
                                    }
                                }
                            }
                        }
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