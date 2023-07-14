use std::sync::Arc;
use gfx::buffer::*;
use gfx::surface::GfxSurface;
use gfx::Gfx;
use maths::vec2::{Vec2u32};

pub fn _demo_objects(_surface: &Arc<dyn GfxSurface>) {
    // GPU Buffer example
    let _test_buffer = Gfx::get().create_buffer(
        "demo_buffer".to_string(),
        &BufferCreateInfo {
            buffer_type: BufferType::Immutable,
            usage: BufferUsage::IndexData,
            access: BufferAccess::Default,
            size: 2048,
        },
    );

    // Framegraph example
    let _res = Vec2u32::new(800, 600);
}
