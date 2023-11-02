use std::path::PathBuf;
use std::sync::Arc;
use core::engine::{App, Builder, Camera, Engine};
use core::world::World;
use ecs::entity::GameObject;
use ecs::query::Query;
use gfx::Gfx;
use gfx::image_sampler::{SamplerCreateInfos};
use gfx::material::Material;
use maths::vec4::Vec4F32;
use plateform::window::{PlatformEvent, WindowCreateInfos};
use renderers::DeferredRenderer;
use resl::ReslShaderInterface;
use shader_base::{BindPoint};
use shader_base::types::{BackgroundColor};

mod gfx_demo;

#[derive(Default)]
pub struct TestApp {
    main_camera: GameObject,
    world: Arc<World>,
}

#[derive(Default)]
struct ProceduralDraw {
    vertices: u32,
    instances: u32,
    material: Option<Arc<Material>>,
}

impl App for TestApp {
    fn pre_initialize(&mut self, _: &mut Builder) {}
    fn initialized(&mut self) {

        // Load data
        let texture = third_party_io::image::read_image_from_file(PathBuf::from("data/textures/cat_stretching.png").as_path()).unwrap();
        let sampler = Gfx::get().create_image_sampler("test sampler".to_string(), SamplerCreateInfos {});
        let shader = ReslShaderInterface::from(PathBuf::from("data/shaders/demo.resl"));

        // Create material
        let mat = Arc::new(Material::default());
        mat.set_shader(shader);
        mat.bind_texture(&BindPoint::new("sTexture"), texture);
        mat.bind_sampler(&BindPoint::new("sSampler"), sampler);

        // Create world
        self.world = Engine::get().new_world();
        self.main_camera = self.world.add_object::<Camera>(Camera {});
        self.world.add_object::<ProceduralDraw>(ProceduralDraw {
            vertices: 3,
            instances: 1,
            material: Some(mat),
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
        let renderer = DeferredRenderer::default();
        renderer.renderer.bind_window_surface(&main_window, &BackgroundColor::Color(Vec4F32::new(0.2, 0.2, 0.0, 1.0)));
        renderer.renderer.set_default_view(&self.main_camera);

        match renderer.renderer.present_node().find_node("g_buffers") {
            None => {}
            Some(g_buffer) => {
                g_buffer.add_render_function(move |ecs, _command_buffer| {
                    Query::<&ProceduralDraw>::new(ecs).for_each(|_| {
                    });
                })
            }
        }

        renderer.renderer.present_node().add_render_function(move |ecs, command_buffer| {
            Query::<&ProceduralDraw>::new(ecs).for_each(|mesh| {
                if let Some(material) =  &mesh.material {
                    material.bind_to(command_buffer);
                    command_buffer.draw_procedural(mesh.vertices, 0, mesh.instances, 0);
                }
            });
        });

        Engine::get().add_renderer(renderer.renderer);
    }

    fn new_frame(&mut self, _delta_seconds: f64) {}
    fn request_shutdown(&self) {}
    fn stopped(&self) {}
}

fn main() {
    let mut engine = Engine::new(TestApp::default());
    engine.start();
}