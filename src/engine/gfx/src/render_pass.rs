use std::sync::Arc;

use maths::vec2::{Vec2F32, Vec2F64};
use maths::vec4::Vec4F32;

use crate::GfxRef;
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
    pub is_present_pass: bool
}

pub trait RenderPass {}


pub struct FrameGraph {}

impl FrameGraph {
    pub fn new(gfx: GfxRef) -> Self {
        Self {}
    }

    pub fn create_or_recreate_swapchain(&self) {}

    pub fn create_or_recreate_render_target(&self) {}

    pub fn get_primary_render_pass(&self) -> Arc<dyn RenderPass> {
        todo!()
    }
}

