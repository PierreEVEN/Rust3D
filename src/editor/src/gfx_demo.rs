use std::path::Path;

use backend_vulkan::GfxVulkan;
use gfx::buffer::*;
use gfx::GfxRef;
use gfx::render_pass::{FrameGraph, RenderPassAttachment, RenderPassCreateInfos};
use gfx::shader::ShaderStage;
use gfx::types::*;
use maths::vec2::{Vec2F32, Vec2u32};
use maths::vec4::Vec4F32;
use shader_compiler::{CompilationResult, CompilerBackend};
use shader_compiler::backends::backend_shaderc::*;
use shader_compiler::parser::Parser;
use shader_compiler::types::*;

pub fn demo_objects(gfx: GfxRef) {
    // GPU Buffer example
    let mut _test_buffer = gfx.create_buffer(&BufferCreateInfo {
        buffer_type: BufferType::Immutable,
        usage: BufferUsage::IndexData,
        access: BufferAccess::Default,
        size: 2048,
        alignment: 16,
        memory_type_bits: 1,
    });


    // Shader program example
    let includer = Box::new(ShaderCIncluder::new());

    let parse_result = match Parser::new(Path::new("./data/shaders/demo.shb"), includer) {
        Ok(result) => {
            println!("successfully parsed shader");
            result
        }
        Err(error) => { panic!("shader syntax error : \n{}", error.to_string()) }
    };

    let shader_compiler = BackendShaderC::new();

    for pass in ["gbuffer".to_string()] {
        let interstage = InterstageData {
            stage_outputs: Default::default(),
            binding_index: 0,
        };

        let null_shader = Vec::new();
        let vertex_code = match parse_result.program_data.get_data(&pass, &ShaderStage::Vertex) {
            Ok(code) => { code }
            Err(error) => {
                println!("failed to get vertex shader code : \n{}", error.to_string());
                &null_shader
            }
        };

        let _sprv = match shader_compiler.compile_to_spirv(vertex_code, ShaderLanguage::HLSL, ShaderStage::Vertex, interstage) {
            Ok(sprv) => {
                println!("compilation succeeded");
                sprv
            }
            Err(error) => {
                println!("shader compilation error : \n{}", error.to_string());
                CompilationResult { binary: vec![] }
            }
        };
    }


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


    let g_buffer_instance = g_buffer_pass.instantiate(res);
    let deferred_combine_instance = deferred_combine_pass.instantiate(res);
}