use std::sync::Arc;

use gfx::buffer::*;
use gfx::GfxRef;
use gfx::render_pass::{RenderPassAttachment, RenderPassCreateInfos};
use gfx::surface::GfxSurface;
use gfx::types::*;
use maths::vec2::{Vec2F32, Vec2u32};
use maths::vec4::Vec4F32;

pub fn _demo_objects(gfx: &GfxRef, surface: &Arc<dyn GfxSurface>) {
    // GPU Buffer example
    let mut _test_buffer = gfx.create_buffer(&BufferCreateInfo {
        buffer_type: BufferType::Immutable,
        usage: BufferUsage::IndexData,
        access: BufferAccess::Default,
        size: 2048,
    });


    // Framegraph example
    let res = Vec2u32::new(800, 600);

    let g_buffer_pass = gfx.create_render_pass(RenderPassCreateInfos {
        name: "GBuffers".to_string(),
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
            }],
        depth_attachment: Some(
            RenderPassAttachment {
                name: "depth".to_string(),
                clear_value: ClearValues::DepthStencil(Vec2F32::new(1.0, 0.0)),
                image_format: PixelFormat::D32_SFLOAT,
            }),
        is_present_pass: false,
    });

    let deferred_combine_pass = gfx.create_render_pass(RenderPassCreateInfos {
        name: "deferred_combine".to_string(),
        color_attachments: vec![RenderPassAttachment {
            name: "color".to_string(),
            clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
            image_format: PixelFormat::R8G8B8A8_UNORM,
        }],
        depth_attachment: None,
        is_present_pass: false,
    });


    let _g_buffer_instance = g_buffer_pass.instantiate(surface, res);
    let _deferred_combine_instance = deferred_combine_pass.instantiate(surface, res);
}