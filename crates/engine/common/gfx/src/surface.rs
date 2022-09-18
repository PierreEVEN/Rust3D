use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use maths::vec2::Vec2u32;
use plateform::window::Window;

use crate::{GfxCast, GfxRef};
use crate::image::GfxImage;
use crate::render_pass::RenderPassInstance;
use crate::types::PixelFormat;

pub struct GfxImageID {
    reference: AtomicU16,
}

impl Clone for GfxImageID {
    fn clone(&self) -> Self {
        GfxImageID { reference: AtomicU16::new(self.reference.load(Ordering::Acquire)) }
    }
}

impl GfxImageID {
    pub fn new(image_index: u8, render_pass_index: u8) -> Self {
        Self {
            reference: AtomicU16::new(image_index as u16 + ((render_pass_index as u16) << 8)),
        }
    }

    pub fn null() -> Self {
        Self {
            reference: AtomicU16::new(0),
        }
    }

    pub fn update(&self, image_index: u8, render_pass_index: u8) {
        self.reference.store(image_index as u16 + ((render_pass_index as u16) << 8), Ordering::Release);
    }

    pub fn image_id(&self) -> u8 {
        self.reference.load(Ordering::Acquire) as u8
    }

    pub fn render_pass_index(&self) -> u8 {
        (self.reference.load(Ordering::Acquire) >> 8) as u8
    }
}

impl PartialEq for GfxImageID {
    fn eq(&self, other: &Self) -> bool {
        self.reference.load(Ordering::Acquire) == other.reference.load(Ordering::Acquire)
    }
}

impl Eq for GfxImageID {}

impl Hash for GfxImageID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(self.reference.load(Ordering::Acquire))
    }
}

impl Display for GfxImageID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("[{}:{}]", self.render_pass_index(), self.image_id()).as_str())
    }
}

pub enum SurfaceAcquireResult {
    Resized,
    Failed(String),
}

pub trait GfxSurface: GfxCast {
    fn create_or_recreate(&self);
    fn get_owning_window(&self) -> &Arc<dyn Window>;
    fn get_surface_pixel_format(&self) -> PixelFormat;
    fn get_image_count(&self) -> u8;
    fn get_current_ref(&self) -> &GfxImageID;
    fn get_surface_texture(&self) -> Arc<dyn GfxImage>;
    fn get_extent(&self) -> Vec2u32;

    fn get_gfx(&self) -> &GfxRef;

    fn acquire(&self, render_pass: &Arc<dyn RenderPassInstance>) -> Result<(), SurfaceAcquireResult>;
    fn submit(&self, render_pass: &Arc<dyn RenderPassInstance>) -> Result<(), SurfaceAcquireResult>;
}

impl dyn GfxSurface {
    pub fn cast<U: GfxSurface + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}