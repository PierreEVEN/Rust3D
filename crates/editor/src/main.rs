use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::fs;
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
use resl::hlsl_builder;
use shader_base::pass_id::PassID;
use shader_base::{ShaderInterface, ShaderStage};

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

pub struct ReslShaderInterface {}

impl From<PathBuf> for ReslShaderInterface {
    fn from(file_path: PathBuf) -> Self {
        let resl_code = fs::read_to_string(file_path.clone()).unwrap();

        let mut builder = hlsl_builder::ReslParser::default();
        match builder.parse(resl_code, file_path.clone()) {
            Ok(_) => {
                //for pass in builder.passes() {}
            }
            Err(err) => {
                match err.token {
                    None => {
                        logger::error!("{}\n  --> {}", err.message, file_path.to_str().unwrap());
                    }
                    Some(token) => {
                        let (line, column) = builder.get_error_location(token);
                        logger::error!("{}\n  --> {}:{}:{}", err.message, file_path.to_str().unwrap(), line, column);
                    }
                }
            }
        };
        Self {}
    }
}

impl ShaderInterface for ReslShaderInterface {
    fn get_spirv_for(&self, _render_pass: &PassID, _stage: ShaderStage) -> Vec<u8> {
        todo!()
    }
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
            None => {}
            Some(program) => { return Some(program.clone()); }
        }
        self.programs.write().unwrap().insert(pass_id.clone(), Gfx::get().create_shader_program(
            "undefined shader program".to_string(),
            pass_id.clone(),
            &ShaderProgramInfos {
                vertex_stage: ShaderProgramStage {
                    spirv: vec![],
                    descriptor_bindings: vec![],
                    push_constant_size: 0,
                    stage_input: vec![],
                },
                fragment_stage: ShaderProgramStage {
                    spirv: vec![],
                    descriptor_bindings: vec![],
                    push_constant_size: 0,
                    stage_input: vec![],
                },
                shader_properties: Default::default(),
            },
        ))
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
        material.set_shader(ReslShaderInterface::from(PathBuf::from("./test.resl")));
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