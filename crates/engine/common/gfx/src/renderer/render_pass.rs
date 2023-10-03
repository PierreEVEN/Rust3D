use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};

use ecs::entity::GameObject;
use maths::vec2::Vec2u32;

use crate::command_buffer::GfxCommandBuffer;
use crate::Gfx;
use crate::image::{GfxImage, ImageType};
use crate::renderer::render_node::RenderNode;
use crate::shader::PassID;
use crate::surface::Frame;
use crate::types::GfxCast;

pub trait RenderPassInstance: GfxCast {
    fn bind(&self, frame: &Frame, context: &RenderPass, res: Vec2u32, command_buffer: &dyn GfxCommandBuffer);
    fn submit(&self, frame: &Frame, context: &RenderPass, command_buffer: &dyn GfxCommandBuffer);
    fn resize(&self, new_size: Vec2u32);
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
    command_buffer: Arc<dyn GfxCommandBuffer>,
}

static mut RENDER_PASSES_TODO: *mut HashMap<PassID, &RenderPass> = std::ptr::null_mut();

impl RenderPass {
    pub fn new(resources: Vec<Arc<dyn GfxImage>>, render_node: &Arc<RenderNode>, initial_res: Vec2u32) -> Self {
        let mut render_pass = Self {
            images: resources,
            res: initial_res,
            inputs: vec![],
            instance: MaybeUninit::uninit(),
            compute_res: render_node.compute_res().clone(),
            source_node: render_node.clone(),
            command_buffer: Gfx::get().create_command_buffer("unnamed".to_string()),
        };

        let instance = Gfx::get().instantiate_render_pass(&render_pass);
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

            match camera.world() {
                None => {}
                Some(ecs) => {
                    if let Ok(mut ecs) = ecs.upgrade().unwrap().write() {
                        self.source_node.draw_content(ecs.deref_mut(), self.command_buffer.as_ref());
                    }
                }
            }

            self.instance.assume_init_ref().submit(frame, self, &*self.command_buffer);
        }
    }

    pub fn resize(&self, new_size: Vec2u32) {
        for input in &self.inputs {
            input.resize(new_size);
        }
        if !self.source_node.is_present_pass()
        {
            for image in &self.images {
                image.resize(ImageType::Texture2d(new_size.x, new_size.y))
            }
        }
        unsafe { self.instance.assume_init_ref().resize(new_size) }
    }

    pub fn images(&self) -> &Vec<Arc<dyn GfxImage>> { &self.images }

    pub fn stringify(&self) -> String {
        let mut dependencies = String::new();

        for dependency in &self.inputs {
            dependencies += format!("\n{}", dependency.stringify().replace('-', "\t-")).as_str()
        }
        if dependencies.is_empty() {
            dependencies = " [Empty]".to_string()
        }

        let mut images = String::new();

        if self.source_node.is_present_pass() {
            images = "[present format]".to_string()
        }
        for image in &self.images {
            images += format!("{:?} ", image.get_format()).as_str()
        }

        format!("\t- name: {}\n\t- images: {}\n\t- initial res: {}x{}\n\t- dependencies:{}\n", self.source_node.get_name(), images, self.res.x, self.res.y, dependencies)
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        todo!()
    }
}
