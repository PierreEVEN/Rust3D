use std::sync::Arc;

use gfx::buffer::*;
use gfx::shader::PassID;
use gfx::surface::GfxSurface;
use gfx::types::*;
use gfx::Gfx;
use maths::vec2::{Vec2f32, Vec2u32};
use maths::vec4::Vec4F32;

pub fn _demo_objects(surface: &Arc<dyn GfxSurface>) {
    // GPU Buffer example
    let mut _test_buffer = Gfx::get().create_buffer(
        "demo_buffer".to_string(),
        &BufferCreateInfo {
            buffer_type: BufferType::Immutable,
            usage: BufferUsage::IndexData,
            access: BufferAccess::Default,
            size: 2048,
        },
    );

    // Framegraph example
    let res = Vec2u32::new(800, 600);
}
