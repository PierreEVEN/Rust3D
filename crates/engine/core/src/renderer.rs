use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock, Weak};
use ecs::entity::GameObject;
use gfx::render_pass::FrameGraph;
use gfx::surface::GfxSurface;

use gfx::types::{ClearValues, PixelFormat};
use logger::fatal;
use maths::vec2::Vec2f32;
use maths::vec4::Vec4F32;
use plateform::window::Window;
use crate::engine::Engine;

pub struct Renderer {
    frame_graphs: RwLock<Vec<FrameGraph>>,
    present_pass: RenderPass,
    camera: RwLock<GameObject>,
}

impl Renderer {
    pub fn add_window(&self, window: &Weak<dyn Window>) {
        self.frame_graphs.write().unwrap().push(self.present_pass.compile_to_surface(Engine::get_mut().new_surface(window)));
    }

    pub fn new_frame(&self) {
        for _frame_graph in &*self.frame_graphs.read().unwrap() {
            todo!("self.framegraph.render(self.camera)");
        }
    }

    pub fn attach_to(&self, camera: &GameObject) {
        *self.camera.write().unwrap() = camera.clone()
    }
}

#[derive(Clone)]
pub struct ResourceColor {
    pub name: String,
    pub clear_value: ClearValues,
    pub image_format: PixelFormat,
}

#[derive(Clone)]
pub struct ResourceDepth {
    pub name: String,
    pub clear_value: ClearValues,
    pub image_format: PixelFormat,
}

#[derive(Clone)]
pub enum Resource {
    Color(ResourceColor),
    Depth(ResourceDepth),
}

impl Hash for Resource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Resource::Color(color_resource) => {
                state.write_u8(128);
                color_resource.image_format.hash(state);
            }
            Resource::Depth(depth_resource) => {
                state.write_u8(255);
                depth_resource.image_format.hash(state);
            }
        }
    }
}

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Resource::Color(color) => {
                if let Resource::Color(other_color) = other { color.image_format == other_color.image_format } else { false }
            }
            Resource::Depth(depth) => {
                if let Resource::Depth(other_depth) = other { depth.image_format == other_depth.image_format } else { false }
            }
        }
    }
}

struct RenderPass {
    name: String,
    inputs: Vec<Arc<RenderPass>>,
    resources: Vec<Resource>,
    present_pass: bool,
}

impl RenderPass {
    pub fn new() -> Self {
        Self {
            name: "RenderPass".to_string(),
            inputs: vec![],
            resources: vec![],
            present_pass: false,
        }
    }

    pub fn present() -> Self {
        Self {
            name: "PresentPass".to_string(),
            inputs: vec![],
            resources: vec![],
            present_pass: true,
        }
    }

    pub fn attach(&mut self, previous: Arc<RenderPass>) {
        if previous.present_pass {
            fatal!("Present pass cannot be attached to parent pass");
        }
        self.inputs.push(previous);
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn add_resource(mut self, resource: Resource) -> Self {
        self.resources.push(resource);
        self
    }

    pub fn compile_to_surface(&self, _surface: Box<dyn GfxSurface>) -> FrameGraph {
        if !self.present_pass { fatal!("Only present pass can be compiled to surface") }
        let mut resources = HashMap::<Resource, ()>::new();
        let mut individual_dependency = self.collect_individual_dependencies();
        
        
        
        
        todo!("surface compilation is not implemented yet")
    }
    
    fn collect_individual_dependencies(&self) -> Vec::<Arc<RenderPass>> {
        let mut result = Vec::<Arc<RenderPass>>::new();
        
        let mut tested_resources = self.inputs.clone();
        for input in &self.inputs {
            let mut other = input.collect_individual_dependencies();
            tested_resources.append(&mut other);
        }
        
        for resource in &tested_resources {
            let mut contains = false;
            for existing in &result {
                if std::ptr::eq(existing.as_ref(), resource.as_ref()) {
                    contains = true;
                    break
                }
            }
            if !contains { result.push(resource.clone()) }
        }
        
        result
    }
}

impl Renderer {
    pub fn default_deferred() -> Self {
        let mut present_pass = RenderPass::present();

        present_pass.attach(Arc::new(RenderPass::new()
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
            ))
        ));

        Self {
            frame_graphs: Default::default(),
            present_pass,
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