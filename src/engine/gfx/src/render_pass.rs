use std::sync::Arc;

use maths::vec2::Vec2u32;
use maths::vec4::Vec4F32;

use crate::{GfxCast, GfxCommandBuffer, GfxRef, GfxSurface, PassID};
use crate::surface::SurfaceAcquireResult;
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
    fn get_pass_id(&self) -> PassID;
}

pub type RenderCallback = fn(&Arc<dyn GfxCommandBuffer>);

pub trait RenderPassInstance: GfxCast {
    fn resize(&self, new_res: Vec2u32);
    fn draw(&self);
    fn on_render(&self, f: RenderCallback);
}

pub struct FrameGraph {
    surface: Arc<dyn GfxSurface>,
    present_pass: Arc<dyn RenderPassInstance>,
}

impl FrameGraph {
    pub fn from_surface(_gfx: &GfxRef, surface: &Arc<dyn GfxSurface>, clear_value: Vec4F32) -> Self {
        let render_pass_ci = RenderPassCreateInfos {
            name: "surface_pass".to_string(),
            color_attachments: vec![RenderPassAttachment {
                name: "color".to_string(),
                clear_value: ClearValues::Color(clear_value),
                image_format: surface.get_surface_pixel_format(),
            }],
            depth_attachment: None,
            is_present_pass: true,
        };

        let res = surface.get_owning_window().get_geometry();

        let draw_pass = _gfx.create_render_pass(render_pass_ci).instantiate(surface, Vec2u32::new(res.width() as u32, res.height() as u32));

        Self {
            surface: surface.clone(),
            present_pass: draw_pass,
        }
    }
    
    pub fn main_pass(&self) -> &Arc<dyn RenderPassInstance> {
        &self.present_pass
    }

    pub fn begin(&self) -> Result<(), String> {
        return match self.surface.acquire(&self.present_pass) {
            Ok(_) => {
                // @TODO : draw children passes
                Ok(self.present_pass.draw())
            }
            Err(error) => {
                match error {
                    SurfaceAcquireResult::Resized => {
                        self.present_pass.resize(self.surface.get_extent());
                        Err("framebuffer resized".to_string())
                    }
                    SurfaceAcquireResult::Failed(error) => {
                        Err(error)
                    }
                }
            }
        };
    }

    pub fn submit(&self) {
        match self.surface.submit(&self.present_pass) {
            Ok(_) => {}
            Err(error) => {
                match error {
                    SurfaceAcquireResult::Resized => {
                        self.present_pass.resize(self.surface.get_extent());
                    }
                    SurfaceAcquireResult::Failed(_error) => { panic!("Failed to submit surface : {_error}") }
                }
            }
        };
    }
}

