use std::sync::{Arc, LockResult, RwLock, Weak};
use ecs::entity::GameObject;
use gfx::renderer::frame_graph::FrameGraph;
use gfx::renderer::render_node::RenderNode;
use gfx::renderer::renderer_resource::{Resource, ResourceColor, ResourceDepth};
use gfx::types::{ClearValues, PixelFormat};
use maths::vec2::Vec2f32;
use maths::vec4::Vec4F32;
use plateform::window::Window;
use crate::engine::Engine;

pub struct Renderer {
    frame_graphs: RwLock<Vec<FrameGraph>>,
    present_node: RenderNode,
    camera: RwLock<GameObject>,
}

impl Renderer {
    /// Compile this renderer to target window surface and add it to the render chain
    pub fn bind_window_surface(&self, window: &Weak<dyn Window>) {
        
        let mut frame_graph = self.present_node.compile_to_surface(Engine::get_mut().new_surface(window));

        logger::debug!("Compiled frame graph :\n\t{}", frame_graph.present_pass().stringify());
        
        self.frame_graphs.write().unwrap().push(frame_graph);
    }
    
    pub fn new_frame(&self) {
        match self.camera.read() {
            Ok(camera) => {
                for _frame_graph in &*self.frame_graphs.read().unwrap() {
                    _frame_graph.execute(&*camera);
                }
            }
            Err(_) => {}
        }
    }

    pub fn set_main_view(&self, camera: &GameObject) {
        *self.camera.write().unwrap() = camera.clone()
    }
}

impl Renderer {
    pub fn default_deferred() -> Self {
        
        // Create G-Buffers
        let g_buffers = RenderNode::default()
            .name("g_buffers")
            .add_resource(Resource::Color(
                ResourceColor {
                    name: "color".to_string(),
                    clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                    image_format: PixelFormat::R8G8B8A8_UNORM,
                }
            ))
            .add_resource(Resource::Depth(
                ResourceDepth {
                    name: "depth".to_string(),
                    clear_value: ClearValues::DepthStencil(Vec2f32::new(0.0, 1.0)),
                    image_format: PixelFormat::D32_SFLOAT,
                }
            ));
        
        // Create present pass
        let mut present_node = RenderNode::present();
        present_node.attach(Arc::new(g_buffers));

        Self {
            frame_graphs: Default::default(),
            present_node,
            camera: Default::default(),
        }
    }
}