use std::sync::Arc;

use shader_base::pass_id::PassID;
use core::gfx::Gfx;
use crate::GfxVulkan;
use crate::renderer::vk_render_pass::VkRenderPass;

pub struct DynamicResourceCTX {
    frame: u8,
    render_pass_id: Option<PassID>,
}

impl Default for DynamicResourceCTX {
    fn default() -> Self {
        Self {
            frame: u8::MAX,
            render_pass_id: None,
        }
    }
}

impl DynamicResourceCTX {
    pub fn new(frame: u8, render_pass_id: PassID) -> Self {
        Self {
            frame,
            render_pass_id: Some(render_pass_id),
        }
    }

    pub fn valid(&self) -> bool {
        return self.frame != u8::MAX && self.render_pass_id.is_some();
    }

    pub fn frame(&self) -> u8 {
        return self.frame;
    }

    pub fn render_pass_id(&self) -> &PassID {
        match &self.render_pass_id {
            None => { logger::fatal!("Render pass id is not valid") }
            Some(pass_id) => { pass_id }
        }
    }

    pub fn render_pass(&self) -> Arc<VkRenderPass> {
        match Gfx::get().cast::<GfxVulkan>().render_pass_pool().find_by_id(self.render_pass_id()) {
            None => {}
            Some(render_pass) => { return render_pass.clone(); }
        }
        logger::fatal!("Render pass does not exists")
    }
}
