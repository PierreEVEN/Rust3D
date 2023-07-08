use std::sync::{Arc, LockResult, RwLock, Weak};
use ecs::entity::GameObject;
use gfx::renderer::render_graph::FrameGraph;
use gfx::renderer::render_node::RenderNode;
use gfx::renderer::renderer_resource::{Resource, ResourceColor, ResourceDepth};
use gfx::types::{ClearValues, PixelFormat};
use maths::vec2::Vec2f32;
use maths::vec4::Vec4F32;
use plateform::window::Window;
use crate::engine::Engine;

pub struct Renderer {
    frame_graphs: RwLock<Vec<FrameGraph>>,
    present_node: RenderNode,
    camera: RwLock<GameObject>,
}

impl Renderer {
    /// Compile this renderer to target window surface and add it to the render chain
    pub fn bind_window_surface(&self, window: &Weak<dyn Window>) {
        self.frame_graphs.write().unwrap().push(self.present_node.compile_to_surface(Engine::get_mut().new_surface(window)));
    }
    
    pub fn new_frame(&self) {
        match self.camera.read() {
            Ok(camera) => {
                for _frame_graph in &*self.frame_graphs.read().unwrap() {
                    _frame_graph.execute(&*camera);
                }
            }
            Err(_) => {}
        }
    }

    pub fn set_main_view(&self, camera: &GameObject) {
        *self.camera.write().unwrap() = camera.clone()
    }
}

impl Renderer {
    pub fn default_deferred() -> Self {
        
        // Create G-Buffers
        let g_buffers = RenderNode::default()
            .name("g_buffers")
            .add_resource(Resource::Color(
                ResourceColor {
                    name: "color".to_string(),
                    clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                    image_format: PixelFormat::R8G8B8A8_UNORM,
                }
            ))
            .add_resource(Resource::Depth(
                ResourceDepth {
                    name: "depth".to_string(),
                    clear_value: ClearValues::DepthStencil(Vec2f32::new(0.0, 1.0)),
                    image_format: PixelFormat::D32_SFLOAT,
                }
            ));
        
        // Create present pass
        let mut present_node = RenderNode::present();
        present_node.attach(Arc::new(g_buffers));

        Self {
            frame_graphs: Default::default(),
            present_node,
            camera: Default::default(),
        }
    }
}




/*
        // Create render pass and pass instances
        let g_buffer_pass = Gfx::get().create_render_pass("gbuffer".to_string(), RenderPassCreateInfos {
            pass_id: PassID::new("deferred_combine"),
            color_attachments: vec![RenderPassAttachment {
                name: "color".to_string(),
                clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                image_format: PixelFormat::R8G8B8A8_UNORM,
            }],
            depth_attachment: None,
            is_present_pass: false,
        });
        let def_combine = g_buffer_pass.instantiate(&main_window_surface, main_window_surface.get_extent());


        // Create framegraph
        let main_framegraph = FrameGraph::from_surface(&main_window_surface, Vec4F32::new(1.0, 0.0, 0.0, 1.0));
        main_framegraph.present_pass().attach(def_combine.clone());

        // Create material
        let demo_material = MaterialAsset::new();
        demo_material.meta_data().set_save_path(Path::new("data/demo_shader"));
        demo_material.meta_data().set_name("demo shader".to_string());
        demo_material.set_shader_code(Path::new("data/shaders/resolve.shb"), fs::read_to_string("data/shaders/resolve.shb").expect("failed to read shader_file"));

        // Create images
        let background_image = read_image_from_file(Path::new("data/textures/cat_stretching.png")).expect("failed to create image");

        // Create sampler
        let generic_image_sampler = Gfx::get().create_image_sampler("bg_image".to_string(), SamplerCreateInfos {});

        // Create material instance
        let surface_combine_shader = demo_material.get_program(&PassID::new("surface_pass")).unwrap().instantiate();
        //surface_combine_shader.bind_texture(&BindPoint::new("ui_result"), &imgui_pass.get_images()[0]);
        surface_combine_shader.bind_texture(&BindPoint::new("scene_result"), &def_combine.get_images()[0]);
        surface_combine_shader.bind_sampler(&BindPoint::new("global_sampler"), &generic_image_sampler);

        let background_shader = demo_material.get_program(&PassID::new("deferred_combine")).unwrap().instantiate();
        background_shader.bind_texture(&BindPoint::new("bg_texture"), &background_image);
        background_shader.bind_sampler(&BindPoint::new("global_sampler"), &generic_image_sampler);

        {
            let surface_shader_instance = surface_combine_shader.clone();
            let demo_material = demo_material.clone();
            main_framegraph.present_pass().on_render(Box::new(move |command_buffer| {
                match demo_material.get_program(&command_buffer.get_pass_id()) {
                    None => { logger::fatal!("failed to find compatible permutation [{}]", command_buffer.get_pass_id()); }
                    Some(program) => {
                        command_buffer.bind_program(&program);
                        command_buffer.bind_shader_instance(&surface_shader_instance);
                        command_buffer.draw_procedural(4, 0, 1, 0);
                    }
                };
            }));
        }

        {
            let shader_2_instance = background_shader.clone();
            def_combine.on_render(Box::new(move |command_buffer| {
                match demo_material.get_program(&command_buffer.get_pass_id()) {
                    None => { logger::fatal!("failed to find compatible permutation [{}]", command_buffer.get_pass_id()); }
                    Some(program) => {
                        command_buffer.bind_program(&program);
                        command_buffer.bind_shader_instance(&shader_2_instance);
                        command_buffer.draw_procedural(4, 0, 1, 0);
                    }
                };
            }));
        }

        Self {
            frame_graph: main_framegraph
        }
    }

    pub fn draw(&self) {
        if self.frame_graph.begin().is_ok() {
            self.frame_graph.submit();
        };
    }
 */