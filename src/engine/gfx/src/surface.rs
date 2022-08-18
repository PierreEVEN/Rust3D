use std::sync::Arc;
use plateform::window::Window;
use crate::{GfxCast, RenderPass, RenderPassCreateInfos};
use crate::image::GfxImage;
use crate::types::PixelFormat;

pub trait GfxSurface: GfxCast {
    fn create_or_recreate(&self);
    fn get_owning_window(&self) -> &Arc<dyn Window>;
    fn get_surface_pixel_format(&self) -> PixelFormat;
    fn get_image_count(&self) -> u8;
    fn get_current_image(&self) -> u8;
    fn get_images(&self) -> Vec<Arc<dyn GfxImage>>;

    fn create_render_pass(&self, create_infos: RenderPassCreateInfos) -> Arc<dyn RenderPass>;

    fn begin(&self) -> Result<(), String>;
    fn submit(&self);
}