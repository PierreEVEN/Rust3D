use std::sync::{Arc, LockResult, RwLock};

use ecs::entity::GameObject;

use crate::image::{GfxImage, ImageType};
use crate::renderer::render_pass::RenderPass;
use crate::surface::{Frame, GfxSurface};

/// This is the standard representation of a frame graph.
/// Then it will be compiled to a representation that fits the running graphic backend.
pub struct FrameGraph {
    present_pass: RenderPass,
    surface: Option<Box<dyn GfxSurface>>,
    frame: Frame,
    max_frames: u8,
}

impl FrameGraph {
    /// Create a framegraph for a given surface
    pub fn new_surface(surface: Box<dyn GfxSurface>, max_frames: u8) -> Self {
        Self { present_pass: RenderPass::new_present_surface(&*surface), surface: Some(surface), frame: Frame::null(), max_frames }
    }

    /// Create a framegraph for a given render target image
    pub fn new_image(_image: Arc<dyn GfxImage>, max_frames: u8) -> Self {
        todo!()
    }

    /// Retrieve the present pass (also the root of the render graph)
    pub fn present_pass(&mut self) -> &mut RenderPass {
        &mut self.present_pass
    }

    /// Render framegraph from given point of view
    pub fn execute(&self, camera: &GameObject) {
        match &self.surface {
            None => {}
            Some(surface) => {
                self.present_pass.draw(&self.frame, surface.get_surface_texture().res_2d(), camera)
            }
        }
        
        self.frame.update((self.frame.image_id() + 1) % self.max_frames, u8::MAX);
    }
}
