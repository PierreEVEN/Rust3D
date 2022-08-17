use std::sync::Arc;
use plateform::window::Window;
use crate::types::PixelFormat;

pub trait GfxSurface {
    fn create_or_recreate(&self);
    fn get_owning_window(&self) -> &Arc<dyn Window>;
    fn get_surface_pixel_format(&self) -> PixelFormat;
    fn get_image_count(&self) -> u8;
    fn get_current_image(&self) -> u8;
    
    fn begin(&self) -> Result<(), String>;
    fn submit(&self);
}