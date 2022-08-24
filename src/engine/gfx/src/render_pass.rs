use std::sync::{Arc};

use maths::vec2::Vec2u32;
use maths::vec4::Vec4F32;

use crate::{GfxCast, GfxRef, GfxSurface};
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
    fn instantiate(&self, surface: &Arc<dyn GfxSurface>, res: Vec2u32) -> Arc<dyn RenderPassInstance>;
    fn get_clear_values(&self) -> &Vec<ClearValues>;
    fn get_config(&self) -> &RenderPassCreateInfos;
}

pub trait RenderPassInstance {
    fn resize(&self, new_res: Vec2u32);
    fn begin(&self);
    fn end(&self);
}

pub struct FrameGraph {
    surface: Arc<dyn GfxSurface>,
    present_pass: Arc<dyn RenderPassInstance>,
}

impl FrameGraph {
    pub fn from_surface(_gfx: &GfxRef, surface: &Arc<dyn GfxSurface>) -> Self {
        let render_pass_ci = RenderPassCreateInfos {
            name: "surface_pass".to_string(),
            color_attachments: vec![RenderPassAttachment {
                name: "color".to_string(),
                clear_value: ClearValues::Color(Vec4F32 { x: 1.0, y: 1.0, z: 0.0, w: 1.0 }),
                image_format: surface.get_surface_pixel_format(),
            }],
            depth_attachment: None,
            is_present_pass: true,
        };

        let res = surface.get_owning_window().get_geometry();

        let draw_pass = surface.create_render_pass(render_pass_ci).instantiate(surface, Vec2u32::new(res.width() as u32, res.height() as u32));

        Self {
            surface: surface.clone(),
            present_pass: draw_pass,
        }
    }

    pub fn begin(&self) -> Result<(), String> {
        match self.surface.acquire(&self.present_pass) {
            Ok(_) => {
                self.present_pass.begin();
            }
            Err(error) => { return Err(error); }
        }
        Ok(())
    }

    pub fn submit(&self) {
        self.present_pass.end();
        self.surface.submit()
    }
}

