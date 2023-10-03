use std::sync::{Arc, RwLock, Weak};

use ecs::entity::GameObject;
use gfx::renderer::frame_graph::FrameGraph;
use gfx::renderer::render_node::RenderNode;
use gfx::renderer::renderer_resource::PassResource;
use gfx::types::{BackgroundColor, PixelFormat};
use maths::vec2::Vec2f32;
use maths::vec4::Vec4F32;
use plateform::window::Window;

use crate::engine::Engine;

pub struct Renderer {
    frame_graphs: RwLock<Vec<FrameGraph>>,
    present_node: Arc<RenderNode>,
    camera: RwLock<GameObject>,
}

impl Renderer {
    /// Compile this renderer to target window surface and add it to the render chain
    pub fn bind_window_surface(&self, window: &Weak<dyn Window>, clear_color: &BackgroundColor) {
        let surface = Engine::get_mut().new_surface(window);
        surface.get_surface_texture().set_background_color(clear_color);
        let mut frame_graph = FrameGraph::new_surface(&self.present_node, surface);
        logger::debug!("Compiled frame graph :\n{}", frame_graph.present_pass().stringify());
        self.frame_graphs.write().unwrap().push(frame_graph);
    }

    pub fn new_frame(&self) {
        if let Ok(camera) = self.camera.read() {
            for _frame_graph in &*self.frame_graphs.read().unwrap() {
                _frame_graph.execute(&camera);
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
        &*self.present_node
    }

    pub fn default_deferred() -> Self {
        // Create G-Buffers
        let g_buffers = RenderNode::default()
            .name("g_buffers")
            .add_resource(PassResource {
                name: "color".to_string(),
                clear_value: BackgroundColor::Color(Vec4F32::new(1.0, 0.0, 0.0, 1.0)),
                format: PixelFormat::R8G8B8A8_UNORM,
            })
            .add_resource(PassResource {
                name: "depth".to_string(),
                clear_value: BackgroundColor::DepthStencil(Vec2f32::new(0.0, 1.0)),
                format: PixelFormat::D32_SFLOAT,
            });

        // Create present pass
        let mut present_node = RenderNode::present();
        present_node.attach(Arc::new(g_buffers));

        Renderer::new(present_node)
    }
}