use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use gfx::renderer::render_node;
use gfx::renderer::render_pass::{RenderPass, RenderPassInstance};
use maths::vec2::Vec2u32;
use crate::renderer::vk_render_pass::VkRenderPass;

/// Store render pass per graph node hash.
/// Avoid duplicating vulkan render pass by hashing render nodes 
#[derive(Default)]
pub struct RenderPassPool {
    render_passes: RwLock<HashMap<Arc<render_node::RenderNode>, Box<VkRenderPass>>>,
}

impl RenderPassPool {
    pub fn instantiate(&self, render_pass: &RenderPass, initial_res: Vec2u32) -> Box<dyn RenderPassInstance> {
        self.find_or_create(render_pass);
        match self.render_passes.read().unwrap().get(render_pass.source()) {
            None => {logger::fatal!("Failed to find render pass")}
            Some(found) => {
                Box::new(found.create_instance(render_pass, initial_res))
            }
        }
    }

    fn find_or_create(&self, render_pass: &RenderPass) {
        if !self.render_passes.read().unwrap().contains_key(render_pass.source()) {
            self.render_passes.write().unwrap().insert(render_pass.source().clone(), Box::new(VkRenderPass::new(render_pass.source())));
        }
    }
}
