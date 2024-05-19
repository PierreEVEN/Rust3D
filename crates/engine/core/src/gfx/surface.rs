use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU16, AtomicU8, Ordering};
use std::sync::Arc;
use plateform::window::Window;
use shader_base::types::GfxCast;
use crate::gfx::image::GfxImage;
use crate::gfx::renderer::render_pass::RenderPassInstance;


pub struct Frame {
    reference: AtomicU8,
}

impl Clone for Frame {
    fn clone(&self) -> Self {
        Frame {
            reference: AtomicU8::new(self.reference.load(Ordering::Acquire)),
        }
    }
}

impl Frame {
    pub fn new(image_index: u8) -> Self {
        Self {
            reference: AtomicU8::new(image_index),
        }
    }

    pub fn null() -> Self {
        Self {
            reference: AtomicU8::new(0),
        }
    }

    pub fn set(&self, frame: &Frame) {
        self.reference.store(
            frame.image_id(),
            Ordering::Release,
        );
    }

    pub fn image_id(&self) -> u8 {
        self.reference.load(Ordering::Acquire)
    }
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.reference.load(Ordering::Acquire) == other.reference.load(Ordering::Acquire)
    }
}

impl Eq for Frame {}

impl Hash for Frame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.reference.load(Ordering::Acquire))
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("[{}]", self.image_id()).as_str())
    }
}

pub enum SurfaceAcquireResult {
    Resized,
    Failed(String),
}

pub trait GfxSurface: GfxCast {
    fn create_or_recreate(&self);
    fn get_owning_window(&self) -> &Arc<dyn Window>;
    fn get_surface_texture(&self) -> Arc<dyn GfxImage>;

    fn acquire(
        &self,
        render_pass: &dyn RenderPassInstance,
        global_frame: &Frame,
    ) -> Result<(Frame), SurfaceAcquireResult>;
    
    fn submit(
        &self,
        render_pass: &dyn RenderPassInstance,
        frame: &Frame
    ) -> Result<(), SurfaceAcquireResult>;
}

impl dyn GfxSurface {
    pub fn cast<U: GfxSurface + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}
