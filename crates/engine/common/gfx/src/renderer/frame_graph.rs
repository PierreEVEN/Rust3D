use std::sync::{Arc};

use ecs::entity::GameObject;
use logger::error;
use maths::vec2::Vec2u32;
use crate::Gfx;

use crate::image::{GfxImage, ImageCreateInfos, ImageParams, ImageType};
use crate::image::ImageUsage::{GpuWriteDestination, Sampling};
use crate::renderer::render_node::RenderNode;
use crate::renderer::render_pass::RenderPass;
use crate::surface::{Frame, GfxSurface, SurfaceAcquireResult};

/// This is the standard representation of a frame graph.
/// Then it will be compiled to a representation that fits the running graphic backend.
pub struct FrameGraph {
    present_pass: RenderPass,
    surface: Option<Box<dyn GfxSurface>>,
    frame: Frame
}

impl FrameGraph {
    /// Create a framegraph for a given set of images
    pub fn new_image(top_node: &Arc<RenderNode>, inputs: Vec<Arc<dyn GfxImage>>) -> Self {
        let initial_res = inputs[0].res_2d();
        let mut present_pass = RenderPass::new(inputs, top_node, initial_res);
        for input in top_node.inputs() {
            present_pass.add_input(Arc::new(Self::compile_node(input, initial_res)));
        }
        
        Self {
            present_pass,
            surface: None,
            frame: Frame::null(),
        }
    }

    /// Create a framegraph for a given surface
    pub fn new_surface(top_node: &Arc<RenderNode>, surface: Box<dyn GfxSurface>) -> Self {
        let initial_res = surface.get_surface_texture().res_2d();
        let mut present_pass = RenderPass::new(vec![surface.get_surface_texture()], top_node, initial_res);
        for input in top_node.inputs() {
            present_pass.add_input(Arc::new(Self::compile_node(input, initial_res)));
        }

        Self {
            present_pass,
            surface: Some(surface),
            frame: Frame::null()
        }
    }

    fn compile_node(node: &Arc<RenderNode>, initial_res: Vec2u32) -> RenderPass {
        let mut images = vec![];

        for resource in node.resources() {
            images.push(
                Gfx::get().create_image(
                    resource.name.clone(),
                    ImageCreateInfos {
                        params: ImageParams {
                            pixel_format: resource.format,
                            image_type: ImageType::Texture2d(initial_res.x, initial_res.y),
                            read_only: false,
                            mip_levels: None,
                            usage: GpuWriteDestination | Sampling,
                            background_color: resource.clear_value,
                        },
                        pixels: None,
                    }));
        }

        let mut pass = RenderPass::new(images, node, initial_res);
        for input in node.inputs() {
            pass.add_input(Arc::new(Self::compile_node(input, initial_res)));
        }
        pass
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
                match surface.acquire(self.present_pass.instance().as_ref()) {
                    Ok(_) => {

                        self.frame.update(surface.get_current_ref().image_id(), 0);
                        self.present_pass.draw(&self.frame, surface.get_surface_texture().res_2d(), camera);
                        match surface.submit(self.present_pass.instance().as_ref()) {
                            Ok(_) => {}
                            Err(err) => {
                                match err {
                                    SurfaceAcquireResult::Resized => {
                                        self.present_pass.resize(surface.get_surface_texture().res_2d());
                                        logger::warning!("failed to submit to surface : Surface resized to {:?}", surface.get_surface_texture().res_2d()) 
                                    }
                                    SurfaceAcquireResult::Failed(message) => { error!("failed to submit to surface : {}", message) }
                                }
                            }
                        };
                    }
                    Err(err) => {
                        match err {
                            SurfaceAcquireResult::Resized => {
                                self.present_pass.resize(surface.get_surface_texture().res_2d());
                                logger::warning!("failed to acquire surface image : Surface resized to {:?}", surface.get_surface_texture().res_2d())
                            }
                            SurfaceAcquireResult::Failed(_) => {}
                        }
                    }
                };
            }
        }
    }
}
