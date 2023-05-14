use std::sync::Arc;

use gfx::buffer::*;
use gfx::render_pass::{RenderPassAttachment, RenderPassCreateInfos};
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

    let g_buffer_pass = Gfx::get().instantiate_render_pass(
        "demo_gbuffer".to_string(),
        RenderPassCreateInfos {
            pass_id: PassID::new("GBuffers"),
            color_attachments: vec![
                RenderPassAttachment {
                    name: "albedo".to_string(),
                    clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                    image_format: PixelFormat::R8G8B8A8_UNORM,
                },
                RenderPassAttachment {
                    name: "roughness_metalness_ao".to_string(),
                    clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                    image_format: PixelFormat::R8G8_UNORM,
                },
                RenderPassAttachment {
                    name: "normal".to_string(),
                    clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                    image_format: PixelFormat::R8G8B8A8_UNORM,
                },
                RenderPassAttachment {
                    name: "velocity".to_string(),
                    clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                    image_format: PixelFormat::R16G16B16A16_SFLOAT,
                },
            ],
            depth_attachment: Some(RenderPassAttachment {
                name: "depth".to_string(),
                clear_value: ClearValues::DepthStencil(Vec2f32::new(1.0, 0.0)),
                image_format: PixelFormat::D32_SFLOAT,
            }),
            is_present_pass: false,
        },
    );

    let deferred_combine_pass = Gfx::get().instantiate_render_pass(
        "demo_deferred_combine".to_string(),
        RenderPassCreateInfos {
            pass_id: PassID::new("deferred_combine"),
            color_attachments: vec![RenderPassAttachment {
                name: "color".to_string(),
                clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                image_format: PixelFormat::R8G8B8A8_UNORM,
            }],
            depth_attachment: None,
            is_present_pass: false,
        },
    );

    let _g_buffer_instance = g_buffer_pass.instantiate(surface, res);
    let _deferred_combine_instance = deferred_combine_pass.instantiate(surface, res);
}
