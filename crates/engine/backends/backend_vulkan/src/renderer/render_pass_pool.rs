use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use gfx::renderer::render_node::RenderNode;
use gfx::renderer::render_pass::{RenderPass};

use crate::renderer::vk_render_pass::VkRenderPass;
use crate::renderer::vk_render_pass_instance::VkRenderPassInstance;

/// Store render pass per graph node hash.
/// Avoid duplicating vulkan render pass by hashing render nodes 
#[derive(Default)]
pub struct RenderPassPool {
    render_passes: RwLock<HashMap<Arc<RenderNode>, Arc<VkRenderPass>>>,
    render_pass_ids: RwLock<HashMap<u64, Arc<RenderNode>>>,
    render_pass_next_id: RwLock<u64>
}

impl RenderPassPool {
    pub fn instantiate(&self, render_pass: &RenderPass) -> VkRenderPassInstance {
        self.create_if_needed(render_pass);
        match self.render_passes.read().unwrap().get(render_pass.source()) {
            None => { logger::fatal!("Failed to find render pass") }
            Some(found) => {
                VkRenderPassInstance::new(found.clone(), render_pass)
            }
        }
    }

    fn create_if_needed(&self, render_pass: &RenderPass) {
        if !self.render_passes.read().unwrap().contains_key(render_pass.source()) {
            self.render_passes.write().unwrap().insert(render_pass.source().clone(), Arc::new(VkRenderPass::new(render_pass)));
            self.render_pass_ids.write().unwrap().insert(*self.render_pass_next_id.read().unwrap(), render_pass.source().clone());
            *self.render_pass_next_id.write().unwrap() += 1;
        }
    }

    pub fn find_by_id(&self, render_pass_id: u64) -> Option<Arc<VkRenderPass>> {
        match self.render_pass_ids.read().unwrap().get(&render_pass_id) {
            None => {}
            Some(render_node) => {
                match self.render_passes.read().unwrap().get(render_node) {
                    None => {}
                    Some(render_pass) => { return Some(render_pass.clone()); }
                }
            }
        }
        None
    }
}
