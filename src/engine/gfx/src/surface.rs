use crate::render_pass::FrameGraph;

pub trait GfxSurface {
    fn create_or_recreate(&self);
    fn get_image_count(&self) -> u8;
    fn get_current_image(&self) -> u8;
    
    fn begin(&self) -> Result<(), String>;
    fn submit(&self);
}