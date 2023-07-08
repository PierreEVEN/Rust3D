use ecs::entity::GameObject;
use crate::image::{GfxImage, ImageType};
use crate::renderer::render_pass::RenderPass;
use crate::surface::GfxSurface;

/// This is the standard representation of a frame graph.
/// Then it will be compiled to a representation that fits the running graphic backend.
pub struct FrameGraph {
    present_pass: RenderPass,
    surface: Option<Box<dyn GfxSurface>>,
}

impl FrameGraph {
    /// Create a framegraph for a given surface
    pub fn new_surface(surface: Box<dyn GfxSurface>) -> Self {
        Self { present_pass: RenderPass::new_present_surface(&*surface), surface: Some(surface) }
    }

    /// Create a framegraph for a given render target image
    pub fn new_image(_image: Box<dyn GfxImage>) -> Self {
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
                self.present_pass.draw(surface.get_surface_texture().res_2d(), camera) }
        }
    }
}
