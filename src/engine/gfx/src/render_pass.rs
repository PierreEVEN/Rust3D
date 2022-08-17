use std::sync::Arc;

use maths::vec2::Vec2u32;

use crate::{GfxCast, GfxRef, GfxSurface};
use crate::render_pass_instance::RenderPassInstance;
use crate::types::{ClearValues, PixelFormat};

pub struct RenderPassAttachment {
    pub name: String,
    pub clear_value: ClearValues,
    pub image_format: PixelFormat,
}

pub struct RenderPassCreateInfos {
    pub name: String,
    pub color_attachments: Vec<RenderPassAttachment>,
    pub depth_attachment: Option<RenderPassAttachment>,
    pub is_present_pass: bool,
}

pub trait RenderPass: GfxCast {
    fn instantiate(&self, res: Vec2u32) -> Box<dyn RenderPassInstance>;
    fn get_clear_values(&self) -> &Vec<ClearValues>;
}


pub struct FrameGraph {
    surface: Arc<dyn GfxSurface>,
}

impl FrameGraph {
    pub fn from_surface(surface: Arc<dyn GfxSurface>) -> Self {
        Self {
            surface
        }
    }

    pub fn begin(&self) -> Result<(), String> {
        self.surface.begin()
    }

    pub fn submit(&self) {
        self.surface.submit()
    }
}

