use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use gfx::renderer::render_node::RenderNode;
use gfx::renderer::render_pass::{RenderPass};
use shader_base::pass_id::PassID;

use crate::renderer::vk_render_pass::VkRenderPass;
use crate::renderer::vk_render_pass_instance::VkRenderPassInstance;

/// Store render pass per graph node hash.
/// Avoid duplicating vulkan render pass by hashing render nodes 
#[derive(Default)]
pub struct RenderPassPool {
    render_passes: RwLock<HashMap<Arc<RenderNode>, Arc<VkRenderPass>>>,
    render_pass_ids: RwLock<HashMap<PassID, Arc<VkRenderPass>>>,
}

impl RenderPassPool {
    pub fn instantiate(&self, render_pass: &RenderPass) -> VkRenderPassInstance {
        match self.create_if_needed(render_pass) {
            None => {}
            Some(found) => {
                return VkRenderPassInstance::new(found.clone(), render_pass);
            }
        };
        match self.render_passes.read().unwrap().get(render_pass.source()) {
            None => { logger::fatal!("Failed to find render pass") }
            Some(found) => {
                VkRenderPassInstance::new(found.clone(), render_pass)
            }
        }
    }

    fn create_if_needed(&self, render_pass: &RenderPass) -> Option<Arc<VkRenderPass>> {
        if !self.render_passes.read().unwrap().contains_key(render_pass.source()) {
            let instanced_render_pass = Arc::new(VkRenderPass::new(render_pass, render_pass.get_id().clone()));
            self.render_passes.write().unwrap().insert(render_pass.source().clone(), instanced_render_pass.clone());
            self.render_pass_ids.write().unwrap().insert(render_pass.get_id().clone(), instanced_render_pass.clone());
            return Some(instanced_render_pass);
        }
        None
    }

    pub fn find_by_id(&self, render_pass_id: &PassID) -> Option<Arc<VkRenderPass>> {
        match self.render_pass_ids.read().unwrap().get(render_pass_id) {
            None => { None }
            Some(render_pass) => { Some(render_pass.clone()) }
        }
    }
}
