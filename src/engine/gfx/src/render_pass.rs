use std::sync::Arc;
use crate::GfxInterface;

pub struct RenderPassCreateInfos {
    pub dependencies: Vec<Arc<Box<dyn RenderPass>>>,
}


pub trait RenderPass {
    
    fn create_or_recreate_framebuffer(&self, width: u32, height: u32) {
        
    }
    
    fn attach_dependency(&self) {
        
    }
    
}

pub struct FrameGraph {
    
}

impl FrameGraph {
    pub fn new(gfx: Arc<dyn GfxInterface>) -> Self {
        Self {
            
        }
    }
    
    pub fn create_or_recreate_swapchain(&self) {
        
    }
    
    pub fn create_or_recreate_render_target(&self) {
        
    }
}





