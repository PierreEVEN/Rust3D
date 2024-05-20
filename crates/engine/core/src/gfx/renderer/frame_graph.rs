use std::sync::{Arc, Condvar, Mutex};

use ecs::entity::GameObject;
use logger::error;
use maths::vec2::Vec2u32;

use crate::gfx::buffer::BufferAccess::Default;
use crate::gfx::Gfx;
use crate::gfx::image::{GfxImage, ImageCreateInfos, ImageParams, ImageType};
use crate::gfx::image::ImageUsage::{GpuWriteDestination, Sampling};
use crate::gfx::renderer::render_node::RenderNode;
use crate::gfx::renderer::render_pass::RenderPass;
use crate::gfx::surface::{Frame, GfxSurface, SurfaceAcquireResult};

/// This is the standard representation of a frame graph.
/// Then it will be compiled to a representation that fits the running graphic backend.
pub struct FrameGraph {
    present_pass: RenderPass,
    surface: Option<Arc<dyn GfxSurface>>,
    test_mut: Arc<Mutex<bool>>,
    test_cond: Arc<Condvar>,
}

impl FrameGraph {
    /// Create a framegraph for a given set of images
    pub fn new_image(top_node: &Arc<RenderNode>, inputs: Vec<Arc<dyn GfxImage>>) -> Self {
        let initial_res = inputs[0].res_2d();
        let mut present_pass = RenderPass::new(inputs, top_node, initial_res);
        for input in top_node.inputs() {
            present_pass.add_input(Arc::new(Self::compile_node(input, initial_res)));
        }
        present_pass.instantiate();
        Self {
            present_pass,
            surface: None,
            test_mut: Arc::new(Mutex::new(true)),
            test_cond: Arc::new(Condvar::new()),
        }
    }

    /// Create a framegraph for a given surface
    pub fn new_surface(top_node: &Arc<RenderNode>, surface: Arc<dyn GfxSurface>) -> Self {
        let initial_res = surface.get_surface_texture().res_2d();
        let mut present_pass = RenderPass::new(vec![surface.get_surface_texture()], top_node, initial_res);
        for input in top_node.inputs() {
            present_pass.add_input(Arc::new(Self::compile_node(input, initial_res)));
        }
        present_pass.instantiate();

        let mutex = Arc::new(Mutex::new(true));
        let cond_var = Arc::new(Condvar::new());
        let mutex_clone = mutex.clone();
        let cond_var_clone = cond_var.clone();

        let surface_clone = surface.clone();

        present_pass.pre_present_callback(move |pass, frame| {
            // Will lock until the previous frame is finished
            
            let mut started = mutex_clone.lock().unwrap();
            while !*started {
                started = cond_var_clone.wait(started).unwrap();
            }
            *started = false;
            logger::warning!("A) RELEASE {frame}");
             
            
            return match surface_clone.acquire(pass.instance().as_ref(), frame) {
                Ok(_) => { // This specific frame is used to determine the swapchain image we will use (which is not necessarily the same than the global frame)
                    true
                }
                Err(err) => {
                    match err {
                        SurfaceAcquireResult::Resized => {
                            pass.resize(surface_clone.get_surface_texture().res_2d());
                            logger::warning!("failed to acquire surface image : Surface resized to {:?}", surface_clone.get_surface_texture().res_2d())
                        }
                        SurfaceAcquireResult::Failed(message) => { error!("failed to submit to surface : {}", message) }
                    }
                    false
                }
            };
        });

        Self {
            present_pass,
            surface: Some(surface),
            test_mut: mutex,
            test_cond: cond_var,
        }
    }

    fn compile_node(node: &Arc<RenderNode>, initial_res: Vec2u32) -> RenderPass {
        let mut images = vec![];

        for resource in node.resources() {
            images.push(
                Gfx::get().create_image(
                    format!("{}_{}", node.get_name(), resource.name),
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
        pass.instantiate();
        pass
    }

    /// Retrieve the present pass (also the root of the render graph)
    pub fn present_pass(&mut self) -> &mut RenderPass {
        &mut self.present_pass
    }

    /// Render framegraph from given point of view
    pub fn execute(&self, camera: &GameObject, global_frame: &Frame) {
        match &self.surface {
            None => {}
            Some(surface) => {
                // @TODO : Remove this temp lock and allow full parallel rendering


                // Will lock until the previous frame is finished
                /*
                {
                    let mut started = self.test_mut.lock().unwrap();
                    while !*started {
                        started = self.test_cond.wait(started).unwrap();
                    }
                    *started = false;
                }
                logger::warning!("A) RELEASE {global_frame}");
                 */
                
                if self.present_pass.draw(&global_frame, surface.get_surface_texture().res_2d(), camera) {
                    logger::warning!("D) SUBMIT {global_frame}");
                    match surface.submit(self.present_pass.instance().as_ref(), &global_frame) {
                        Ok(_) => {}
                        Err(err) => {
                            match err {
                                SurfaceAcquireResult::Resized => {
                                    self.present_pass.resize(surface.get_surface_texture().res_2d());
                                    logger::warning!("failed to submit surface image : Surface resized to {:?}", surface.get_surface_texture().res_2d())
                                }
                                SurfaceAcquireResult::Failed(message) => { error!("failed to submit to surface : {}", message) }
                            }
                        }
                    };
                }
                logger::warning!("E) FINISHED {global_frame}");
                *self.test_mut.lock().unwrap() = true;
                self.test_cond.notify_one();
            }
        }
    }
}
