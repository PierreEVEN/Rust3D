use std::mem::MaybeUninit;
use std::sync::{Arc, RwLock};
use ecs::entity::GameObject;
use maths::vec2::Vec2u32;
use crate::command_buffer::GfxCommandBuffer;
use crate::Gfx;
use crate::image::GfxImage;
use crate::renderer::render_node::RenderNode;
use crate::surface::{Frame, GfxSurface};
use crate::types::{ClearValues, GfxCast};

pub trait RenderPassInstance: GfxCast {
    fn bind(&self, frame: &Frame, context: &RenderPass, res: Vec2u32, command_buffer: &dyn GfxCommandBuffer);
    fn submit(&self, frame: &Frame, context: &RenderPass, command_buffer: &dyn GfxCommandBuffer);
}

impl dyn RenderPassInstance {
    pub fn cast<U: RenderPassInstance + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}

/// Compiling a RenderNode result in a RenderPass.
/// Each render pass contains a RenderPassInstance which will be implemented depending on how the backend works
pub struct RenderPass {
    images: Vec<Arc<dyn GfxImage>>,
    res: Vec2u32,
    inputs: Vec<Arc<RenderPass>>,
    instance: MaybeUninit<Box<dyn RenderPassInstance>>,
    compute_res: Arc<RwLock<dyn FnMut(Vec2u32) -> Vec2u32>>,
    source_node: Arc<RenderNode>,
    clear_values: Vec<ClearValues>,
    command_buffer: Arc<dyn GfxCommandBuffer>,
}

impl RenderPass {
    pub fn new(resources: Vec<Arc<dyn GfxImage>>, render_node: &Arc<RenderNode>, initial_res: Vec2u32) -> Self {
        
        let mut clear_values = vec![];
        for color in &render_node.color_resources() {
            clear_values.push(color.clear_value);
        }
        for depth in &render_node.depth_resource() {
            clear_values.push(depth.clear_value);
        }
        
        let mut render_pass = Self {
            images: resources,
            res: Default::default(),
            inputs: vec![],
            instance: MaybeUninit::uninit(),
            compute_res: render_node.compute_res().clone(),
            source_node: render_node.clone(),
            clear_values,
            command_buffer: Gfx::get().create_command_buffer("unnamed".to_string()),
        };

        let instance = Gfx::get().instantiate_render_pass(&render_pass, initial_res);
        render_pass.instance = MaybeUninit::new(instance);

        render_pass
    }

    pub fn new_present_surface(_surface: &dyn GfxSurface) -> Self {
        let mut render_pass = Self {
            images: vec![],
            res: Default::default(),
            inputs: vec![],
            instance: MaybeUninit::uninit(),
            compute_res: Arc::new(RwLock::new(|res| { res })),
            source_node: Arc::new(RenderNode::present()),
            clear_values: vec![ClearValues::DontClear],
            command_buffer: Gfx::get().create_command_buffer("unnamed".to_string()),
        };

        let instance = Gfx::get().instantiate_render_pass(&render_pass, _surface.get_surface_texture().res_2d());
        render_pass.instance = MaybeUninit::new(instance);

        render_pass
    }

    pub fn source(&self) -> &Arc<RenderNode> {
        &self.source_node
    }

    pub fn add_input(&mut self, input: Arc<RenderPass>) {
        self.inputs.push(input);
    }
    
    pub fn inputs(&self) -> &Vec<Arc<RenderPass>> {
        &self.inputs
    }

    pub fn instance(&self) -> &Box<dyn RenderPassInstance> {
        unsafe { self.instance.assume_init_ref() }
    }
    
    pub fn draw(&self, frame: &Frame, res: Vec2u32, camera: &GameObject) {
        //TODO parallelize
        for input in &self.inputs {
            input.draw(frame, res, camera);
        }
        unsafe {
            self.instance.assume_init_ref().bind(frame, self, (*self.compute_res.write().unwrap())(res), &*self.command_buffer);
            logger::info!("draw content here");
            self.instance.assume_init_ref().submit(frame, self, &*self.command_buffer);
        }
    }
    
    pub fn clear_values(&self) -> &Vec<ClearValues> {
        &self.clear_values
    }
    
    pub fn images(&self) -> &Vec<Arc<dyn GfxImage>> {&self.images}
    
    pub fn stringify(&self) -> String {
        let mut inputs = String::new();
        
        for input in &self.inputs {
            inputs += format!("{}\n", input.stringify()).as_str()
        }
        
        format!("initial res : {}x{}\ninputs : {}", self.res.x, self.res.y, inputs)
    }
}
