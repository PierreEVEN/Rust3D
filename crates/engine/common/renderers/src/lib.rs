use std::sync::Arc;
use core::gfx::renderer::render_node::RenderNode;
use core::gfx::renderer::renderer_resource::PassResource;
use core::gfx::renderer::renderer::Renderer;
use imgui::ImGUiContext;
use maths::vec2::Vec2f32;
use maths::vec4::Vec4F32;
use shader_base::types::{BackgroundColor, PixelFormat};

pub struct DeferredRenderer {
    pub renderer: Renderer,
    pub imgui_context: ImGUiContext,
}

impl Default for DeferredRenderer {
    fn default() -> Self {
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

        let imgui_context = ImGUiContext::new("imgui backend".to_string());

        // Create present pass
        let mut present_node = RenderNode::present();
        present_node.attach(Arc::new(g_buffers));
        present_node.attach(imgui_context.render_node.clone());
        
        Self {
            renderer : Renderer::new(present_node),
            imgui_context
        }
    }
}