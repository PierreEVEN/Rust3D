use std::sync::Arc;
use gfx::Gfx;

use crate::GfxVulkan;
use crate::renderer::vk_render_pass::VkRenderPass;

pub struct DynamicResourceCTX {
    frame: u8,
    render_pass_id: u64,
}

impl Default for DynamicResourceCTX {
    fn default() -> Self {
        Self::new(u8::MAX, u64::MAX)
    }
}

impl DynamicResourceCTX {
    pub fn new(frame: u8, render_pass_id: u64) -> Self {
        Self {
            frame,
            render_pass_id,
        }
    }

    pub fn valid(&self) -> bool {
        return self.frame != u8::MAX && self.render_pass_id != u64::MAX;
    }

    pub fn frame(&self) -> u8 {
        return self.frame;
    }

    pub fn render_pass_id(&self) -> u64 {
        self.render_pass_id
    }
    
    pub fn render_pass(&self) -> Arc<VkRenderPass> {
        match Gfx::get().cast::<GfxVulkan>().render_pass_pool().find_by_id(self.render_pass_id) {
            None => {}
            Some(render_pass) => { return render_pass; }
        }
        logger::fatal!("Render pass does not exists")
    }
}
