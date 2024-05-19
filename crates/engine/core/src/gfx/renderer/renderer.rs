use std::sync::{Arc, RwLock, Weak};

use ecs::entity::GameObject;
use plateform::window::Window;
use shader_base::types::{BackgroundColor};

use crate::engine::Engine;
use crate::gfx::command_buffer::CommandCtx;
use crate::gfx::Gfx;
use crate::gfx::renderer::frame_graph::FrameGraph;
use crate::gfx::renderer::render_node::RenderNode;
use crate::gfx::surface::Frame;

pub struct Renderer {
    frame_graphs: RwLock<Vec<FrameGraph>>,
    present_node: Arc<RenderNode>,
    camera: RwLock<GameObject>,
}

impl Renderer {
    /// Compile this renderer to target window surface and add it to the render chain
    pub fn bind_window_surface(&self, window: &Weak<dyn Window>, clear_color: &BackgroundColor) {
        let surface = Gfx::get_mut().new_surface(window);
        surface.get_surface_texture().set_background_color(clear_color);
        let mut frame_graph = FrameGraph::new_surface(&self.present_node, surface);
        logger::debug!("Compiled frame graph :\n{}", frame_graph.present_pass().stringify());
        self.frame_graphs.write().unwrap().push(frame_graph);
    }

    pub fn new_frame(&self, global_frame: &Frame) {
        if let Ok(camera) = self.camera.read() {
            for frame_graph in &*self.frame_graphs.read().unwrap() {
                frame_graph.execute(&camera, global_frame);
            }
        }
    }

    pub fn set_default_view(&self, camera: &GameObject) {
        *self.camera.write().unwrap() = camera.clone()
    }
}

impl Renderer {
    pub fn new(present_node: RenderNode) -> Self {
        Self {
            frame_graphs: Default::default(),
            present_node: Arc::new(present_node),
            camera: Default::default(),
        }
    }

    pub fn present_node(&self) -> &RenderNode {
        &self.present_node
    }
}