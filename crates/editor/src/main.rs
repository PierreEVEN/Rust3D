use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use core::engine::{App, Builder, Camera, Engine};
use core::renderer::Renderer;
use core::world::World;
use ecs::entity::GameObject;
use ecs::query::Query;
use gfx::Gfx;
use gfx::mesh::Mesh;
use gfx::shader::{ShaderProgram, ShaderProgramInfos, ShaderProgramStage};
use gfx::shader_instance::ShaderInstance;
use gfx::types::BackgroundColor;
use maths::vec4::Vec4F32;
use plateform::window::{PlatformEvent, WindowCreateInfos};
use resl::ReslShaderInterface;
use shader_base::pass_id::PassID;
use shader_base::{CompilationError, ShaderInterface, ShaderStage};

mod gfx_demo;

#[derive(Default)]
pub struct TestApp {
    main_camera: GameObject,
    world: Arc<World>,
}

#[derive(Default)]
struct MeshComponent {
    _mesh: Option<Arc<Mesh>>,
    _material: Option<Arc<dyn ShaderInstance>>,
    _val: u32,
    _name: String,
}

#[derive(Default)]
struct Material {
    programs: RwLock<HashMap<PassID, Arc<dyn ShaderProgram>>>,
    shader_interface: RwLock<Option<Box<dyn ShaderInterface>>>,
}

impl Material {
    pub fn set_shader<T: 'static + ShaderInterface>(&mut self, shader: T) {
        *self.shader_interface.write().unwrap() = Some(Box::new(shader));
    }

    pub fn get_program_for_pass(&self, pass_id: &PassID) -> Option<Arc<dyn ShaderProgram>> {
        match self.programs.read().unwrap().get(pass_id) {
            None => {
                if let Some(shi) = &*self.shader_interface.read().unwrap() {
                    return self.programs.write().unwrap().insert(pass_id.clone(), Gfx::get().create_shader_program(
                        "undefined shader program".to_string(),
                        pass_id.clone(), shi.as_ref(),
                    ));
                }
                return None;
            }
            Some(program) => { return Some(program.clone()); }
        }
    }
}

impl App for TestApp {
    fn pre_initialize(&mut self, _: &mut Builder) {}
    fn initialized(&mut self) {
        // Create world
        self.world = Engine::get().new_world();
        self.main_camera = self.world.add_object::<Camera>(Camera {});
        self.world.add_object::<MeshComponent>(MeshComponent { _mesh: None, _material: None, _val: 5, _name: "raw test mesh".to_string() });

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

        let mut material = Material::default();
        material.set_shader(ReslShaderInterface::from(PathBuf::from("data/shaders/demo.resl")));
        let mat = Arc::new(material);

        match renderer.present_node().find_node("g_buffers") {
            None => {}
            Some(g_buffer) => {
                g_buffer.add_render_function(move |ecs, command_buffer| {
                    let mat = mat.clone();
                    Query::<&mut MeshComponent>::new(ecs).for_each(|_| {
                        let program = mat.get_program_for_pass(&command_buffer.get_pass_id()).unwrap();
                        command_buffer.bind_program(&program);
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

fn main() {
    let mut engine = Engine::new(TestApp::default());
    engine.start();
}