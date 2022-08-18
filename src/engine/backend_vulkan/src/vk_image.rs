use ash::vk::{Image, ImageView};
use gfx::image::GfxImage;

pub struct VkImage {
    pub image: Image,
    pub view: ImageView,
}

impl GfxImage for VkImage {
    
}

impl VkImage {
    
}