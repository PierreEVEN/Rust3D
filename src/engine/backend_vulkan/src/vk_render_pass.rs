use std::sync::Arc;
use gfx::render_pass::RenderPass;

pub struct VkRenderPass {
    
}

impl RenderPass for VkRenderPass {
    
}


impl VkRenderPass {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            
        })
    }
}