use maths::vec2::Vec2u32;
use crate::render_pass::FrameGraph;

pub struct SurfaceCreateInfos {
    pub image_count: u32,
    pub extent: Vec2u32,
}


pub trait GfxSurface {
    fn create_or_recreate(&self, create_infos: SurfaceCreateInfos);
    fn get_image_count(&self) -> u8;
    fn get_current_image(&self) -> u8;
    
    fn begin(&self);
    fn submit(&self);
}