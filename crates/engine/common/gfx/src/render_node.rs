use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::sync::{Arc, RwLock};
use ecs::entity::GameObject;
use logger::{fatal, info};
use maths::vec2::Vec2u32;
use crate::Gfx;
use crate::image::{GfxImage, ImageCreateInfos, ImageParams, ImageType};
use crate::image::ImageUsage::GpuWriteDestination;
use crate::render_node::Resource::{Color, Depth};
use crate::surface::GfxSurface;
use crate::types::{ClearValues, PixelFormat};

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
            Color(color_resource) => {
                state.write_u8(128);
                color_resource.image_format.hash(state);
            }
            Depth(depth_resource) => {
                state.write_u8(255);
                depth_resource.image_format.hash(state);
            }
        }
    }
}

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Color(color) => {
                if let Color(other_color) = other { color.image_format == other_color.image_format } else { false }
            }
            Depth(depth) => {
                if let Depth(other_depth) = other { depth.image_format == other_depth.image_format } else { false }
            }
        }
    }
}

pub trait RenderPassInstance {
    fn init(&self, context: &RenderPass);
    fn bind(&self, context: &RenderPass, res: Vec2u32);
    fn submit(&self, context: &RenderPass);
}

pub struct RenderPass {
    images: Vec<Arc<dyn GfxImage>>,
    res: Vec2u32,
    inputs: Vec<Arc<RenderPass>>,
    instance: MaybeUninit<Box<dyn RenderPassInstance>>,
    compute_res: Arc<RwLock<dyn FnMut(Vec2u32) -> Vec2u32>>,
    source_node: Arc<RenderNode>,
}

impl RenderPass {
    pub fn new(resources: Vec<Arc<dyn GfxImage>>, render_node: &Arc<RenderNode>, initial_res: Vec2u32) -> Self {
        let mut render_pass = Self {
            images: resources,
            res: Default::default(),
            inputs: vec![],
            instance: MaybeUninit::uninit(),
            compute_res: render_node.compute_res.clone(),
            source_node: render_node.clone(),
        };

        let instance = Gfx::get().instantiate_render_pass(&render_pass, initial_res);
        instance.init(&render_pass);
        render_pass.instance = MaybeUninit::new(instance);

        render_pass
    }

    pub fn new_present_surface(_surface: &Box<dyn GfxSurface>) -> Self {
        let mut render_pass = Self {
            images: vec![],
            res: Default::default(),
            inputs: vec![],
            instance: MaybeUninit::uninit(),
            compute_res: Arc::new(RwLock::new(|res| { res })),
            source_node: Arc::new(RenderNode::present()),
        };

        let instance = Gfx::get().instantiate_render_pass(&render_pass, _surface.get_extent());
        instance.init(&render_pass);
        render_pass.instance = MaybeUninit::new(instance);

        render_pass
    }

    pub fn source(&self) -> &Arc<RenderNode> {
        &self.source_node
    }

    pub fn add_input(&mut self, input: Arc<RenderPass>) {
        self.inputs.push(input);
    }

    pub fn draw(&self, res: Vec2u32, camera: &GameObject) {
        for input in &self.inputs {
            input.draw(res, camera);
        }
        unsafe {
            self.instance.assume_init_ref().bind(self, (*self.compute_res.write().unwrap())(res));
            info!("draw content here");
            self.instance.assume_init_ref().submit(self);
        }
    }
}

pub struct FrameGraph {
    present_pass: RenderPass,
    surface: Option<Box<dyn GfxSurface>>,
}

impl FrameGraph {
    pub fn new_surface(surface: Box<dyn GfxSurface>) -> Self {
        Self { present_pass: RenderPass::new_present_surface(&surface), surface: Some(surface) }
    }

    pub fn present_pass(&mut self) -> &mut RenderPass {
        &mut self.present_pass
    }

    pub fn execute(&self, camera: &GameObject) {
        match &self.surface {
            None => {}
            Some(surface) => { self.present_pass.draw(surface.get_extent(), camera) }
        }
    }
}

pub struct RenderNode {
    name: String,
    inputs: Vec<Arc<RenderNode>>,
    resources: Vec<Resource>,
    compute_res: Arc<RwLock<dyn FnMut(Vec2u32) -> Vec2u32>>,
    present_pass: bool,
}

impl Default for RenderNode {
    fn default() -> Self {
        Self {
            name: "RenderPass".to_string(),
            inputs: vec![],
            resources: vec![],
            compute_res: Arc::new(RwLock::new(|res| { res })),
            present_pass: false,
        }
    }
}

impl RenderNode {
    pub fn color_resources(&self) -> Vec<ResourceColor> {
        let mut resources = vec![];
        for resource in &self.resources {
            if let Color(color) = resource {
                resources.push(color.clone());
            }
        }
        resources
    }

    pub fn depth_resource(&self) -> Vec<ResourceDepth> {
        let mut resources = vec![];
        for resource in &self.resources {
            if let Depth(depth) = resource {
                resources.push(depth.clone());
            }
        }
        resources
    }

    pub fn is_present_pass(&self) -> bool { self.present_pass }

    pub fn present() -> Self {
        Self {
            name: "PresentPass".to_string(),
            inputs: vec![],
            resources: vec![],
            compute_res: Arc::new(RwLock::new(|res| { res })),
            present_pass: true,
        }
    }

    pub fn attach(&mut self, previous: Arc<RenderNode>) {
        if previous.present_pass {
            fatal!("Present pass cannot be attached to parent pass");
        }
        self.inputs.push(previous);
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn res_override<H: FnMut(Vec2u32) -> Vec2u32 + 'static>(mut self, func: H) {
        self.compute_res = Arc::new(RwLock::new(func));
    }

    pub fn add_resource(mut self, resource: Resource) -> Self {
        self.resources.push(resource);
        self
    }

    fn compile_item(instance: &mut RenderPass, render_passes: &RenderNode, initial_res: Vec2u32) {
        for input in &render_passes.inputs {
            let mut images = vec![];

            for resource in &input.resources {
                images.push(Gfx::get().create_image(
                    "undefined name".to_string(),
                    ImageCreateInfos { params: ImageParams {
                        pixel_format: resource.,
                        image_type: ImageType::Texture2d(initial_res.x, initial_res.y),
                        read_only: false,
                        mip_levels: None,
                        usage: GpuWriteDestination,
                    }, pixels: None },
                ))
            }


            let mut new_instance = RenderPass::new(images, input, initial_res);
            Self::compile_item(&mut new_instance, input, initial_res);
            instance.add_input(Arc::new(new_instance));
        }
    }


    pub fn compile_to_surface(&self, surface: Box<dyn GfxSurface>) -> FrameGraph {
        if !self.present_pass { fatal!("Only present pass can be compiled to surface") }
        let initial_res = surface.get_extent();
        let mut framegraph = FrameGraph::new_surface(surface);
        Self::compile_item(framegraph.present_pass(), self, initial_res);
        framegraph
    }
}

impl Hash for RenderNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.resources.hash(state);
    }
}

impl PartialEq for RenderNode {
    fn eq(&self, other: &Self) -> bool {
        self.resources == other.resources
    }
}

impl Eq for RenderNode {}
