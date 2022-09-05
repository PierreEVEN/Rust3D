use std::fs;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr::null_mut;
use std::sync::Arc;

use memoffset::offset_of;

use gfx::buffer::BufferMemory;
use gfx::command_buffer::GfxCommandBuffer;
use gfx::GfxRef;
use gfx::image::{GfxImage, GfxImageUsageFlags, ImageCreateInfos, ImageParams, ImageUsage};
use gfx::image::ImageType::Texture2d;
use gfx::image_sampler::{ImageSampler, SamplerCreateInfos};
use gfx::render_pass::{GraphRenderCallback, RenderPass, RenderPassAttachment, RenderPassCreateInfos, RenderPassInstance};
use gfx::shader::{PassID, ShaderLanguage, ShaderProgram, ShaderProgramInfos, ShaderProgramStage, ShaderPropertyType, ShaderStage, ShaderStageInput};
use gfx::shader_instance::{BindPoint, ShaderInstance, ShaderInstanceCreateInfos};
use gfx::surface::GfxSurface;
use gfx::types::{ClearValues, PixelFormat};
use imgui_bindings::{igCreateContext, igEndFrame, igGetDrawData, igGetIO, igGetMainViewport, igGetStyle, igNewFrame, igRender, igStyleColorsDark, ImDrawVert, ImFontAtlas_GetTexDataAsRGBA32, ImGuiBackendFlags__ImGuiBackendFlags_HasMouseCursors, ImGuiBackendFlags__ImGuiBackendFlags_HasSetMousePos, ImGuiBackendFlags__ImGuiBackendFlags_PlatformHasViewports, ImGuiConfigFlags__ImGuiConfigFlags_DockingEnable, ImGuiConfigFlags__ImGuiConfigFlags_NavEnableGamepad, ImGuiConfigFlags__ImGuiConfigFlags_NavEnableKeyboard, ImGuiConfigFlags__ImGuiConfigFlags_ViewportsEnable, ImGuiContext, ImTextureID, ImVec2};
use maths::vec2::Vec2F32;
use maths::vec4::Vec4F32;
use shader_compiler::backends::backend_shaderc::{BackendShaderC, ShaderCIncluder};
use shader_compiler::CompilerBackend;
use shader_compiler::parser::Parser;
use shader_compiler::types::InterstageData;

pub struct ImGUiContext {
    pub font_texture: Arc<dyn GfxImage>,
    pub shader_program: Arc<dyn ShaderProgram>,
    pub shader_instance: Arc<dyn ShaderInstance>,
    pub image_sampler: Arc<dyn ImageSampler>,
    pub render_pass: Arc<dyn RenderPass>,
    pub context: *mut ImGuiContext,
    _gfx: GfxRef,
}


impl ImGUiContext {
    pub fn new(gfx: &GfxRef) -> Arc<Self> {
        let imgui_context = unsafe { igCreateContext(null_mut()) };

        let io = unsafe { &mut *igGetIO() };
        io.ConfigFlags |= ImGuiConfigFlags__ImGuiConfigFlags_NavEnableKeyboard;
        io.ConfigFlags |= ImGuiConfigFlags__ImGuiConfigFlags_NavEnableGamepad;
        io.ConfigFlags |= ImGuiConfigFlags__ImGuiConfigFlags_DockingEnable;
        io.ConfigFlags |= ImGuiConfigFlags__ImGuiConfigFlags_ViewportsEnable;

        io.BackendPlatformUserData = null_mut();
        io.BackendPlatformName = "imgui backend".as_ptr() as *const c_char;
        io.BackendFlags |= ImGuiBackendFlags__ImGuiBackendFlags_HasMouseCursors;
        io.BackendFlags |= ImGuiBackendFlags__ImGuiBackendFlags_HasSetMousePos;
        io.BackendFlags |= ImGuiBackendFlags__ImGuiBackendFlags_PlatformHasViewports;

        let style = unsafe { &mut *igGetStyle() };
        unsafe { igStyleColorsDark(igGetStyle()) };
        style.WindowRounding = 0.0;
        style.ScrollbarRounding = 0.0;
        style.TabRounding = 0.0;
        style.WindowBorderSize = 1.0;
        style.PopupBorderSize = 1.0;
        style.WindowTitleAlign.x = 0.5;
        style.FramePadding.x = 6.0;
        style.FramePadding.y = 6.0;
        style.WindowPadding.x = 4.0;
        style.WindowPadding.y = 4.0;
        style.GrabMinSize = 16.0;
        style.ScrollbarSize = 20.0;
        style.IndentSpacing = 30.0;

        let main_viewport = unsafe { &mut *igGetMainViewport() };
        main_viewport.PlatformHandle = null_mut();

        let mut pixels = null_mut();
        let mut width: i32 = 0;
        let mut height: i32 = 0;
        unsafe { ImFontAtlas_GetTexDataAsRGBA32(io.Fonts, &mut pixels, &mut width, &mut height, null_mut()) }
        let data_size = width * height * PixelFormat::R8G8B8A8_UNORM.type_size() as i32;

        let font_texture = gfx.create_image(ImageCreateInfos {
            params: ImageParams {
                pixel_format: PixelFormat::R8G8B8A8_UNORM,
                image_type: Texture2d(width as u32, height as u32),
                read_only: true,
                mip_levels: None,
                usage: GfxImageUsageFlags::from_flag(ImageUsage::Sampling),
            },
            pixels: Some(unsafe { Vec::from_raw_parts(pixels, data_size as usize, data_size as usize) }),
        });
        unsafe { (&mut *io.Fonts).TexID = font_texture.__static_view_handle() as ImTextureID; }

        let shader_path = String::from("data/shaders/imgui_material.shb");
        let shader_text = match fs::read_to_string(shader_path.clone()) {
            Ok(file_data) => { file_data }
            Err(_) => { panic!("failed to read imgui shader file") }
        };
        let parse_result = Parser::new(&shader_text, &shader_path, Box::new(ShaderCIncluder::new()));
        let imgui_parser_result = match parse_result {
            Ok(result) => {
                result
            }
            Err(error) => {
                panic!("imgui shader syntax error : \n{}", error.to_string())
            }
        };

        let imgui_pass_id = PassID::new("imgui_render_pass");
        let imgui_render_pass = gfx.create_render_pass(RenderPassCreateInfos {
            name: "imgui_render_pass".to_string(),
            color_attachments: vec![RenderPassAttachment {
                name: "color".to_string(),
                clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                image_format: PixelFormat::R8G8B8A8_UNORM,
            }],
            depth_attachment: Some(RenderPassAttachment {
                name: "depth".to_string(),
                clear_value: ClearValues::DepthStencil(Vec2F32::new(1.0, 0.0)),
                image_format: PixelFormat::D24_UNORM_S8_UINT,
            }),
            is_present_pass: false,
        });
        let vertex_data = match imgui_parser_result.program_data.get_data(&imgui_pass_id, &ShaderStage::Vertex) {
            Ok(data) => { data }
            Err(_) => { panic!("failed to get vertex data"); }
        };
        let fragment_data = match imgui_parser_result.program_data.get_data(&imgui_pass_id, &ShaderStage::Fragment) {
            Ok(data) => { data }
            Err(_) => { panic!("failed to get fragment data"); }
        };

        let shader_backend = BackendShaderC::new();

        let vertex_sprv = match shader_backend.compile_to_spirv(vertex_data, Path::new(shader_path.as_str()), ShaderLanguage::HLSL, ShaderStage::Vertex, InterstageData {
            stage_outputs: Default::default(),
            binding_index: 0,
        }) {
            Ok(sprv) => { sprv }
            Err(error) => {
                panic!("Failed to compile vertex shader : \n{}", error.to_string());
            }
        };

        let fragment_sprv = match shader_backend.compile_to_spirv(fragment_data, Path::new(shader_path.as_str()), ShaderLanguage::HLSL, ShaderStage::Fragment, InterstageData {
            stage_outputs: Default::default(),
            binding_index: 0,
        }) {
            Ok(sprv) => { sprv }
            Err(error) => {
                panic!("Failed to compile fragment shader : \n{}", error.to_string());
            }
        };

        let shader_program = gfx.create_shader_program(&imgui_render_pass, &ShaderProgramInfos {
            vertex_stage: ShaderProgramStage {
                spirv: vertex_sprv.binary,
                descriptor_bindings: vertex_sprv.bindings,
                push_constant_size: vertex_sprv.push_constant_size,
                stage_input: vec![ShaderStageInput {
                    location: 0,
                    offset: offset_of!(ImDrawVert, pos) as u32,
                    property_type: ShaderPropertyType { format: PixelFormat::R32G32_SFLOAT },
                }, ShaderStageInput {
                    location: 1,
                    offset: offset_of!(ImDrawVert, uv) as u32,
                    property_type: ShaderPropertyType { format: PixelFormat::R32G32_SFLOAT },
                }, ShaderStageInput {
                    location: 2,
                    offset: offset_of!(ImDrawVert, col) as u32,
                    property_type: ShaderPropertyType { format: PixelFormat::R8G8B8A8_UNORM },
                }],
            },
            fragment_stage: ShaderProgramStage {
                spirv: fragment_sprv.binary,
                descriptor_bindings: fragment_sprv.bindings,
                push_constant_size: fragment_sprv.push_constant_size,
                stage_input: vec![],
            },
            shader_properties: Default::default(),
        });

        let image_sampler = gfx.create_image_sampler(SamplerCreateInfos {});

        let shader_instance = gfx.create_shader_instance(ShaderInstanceCreateInfos { bindings: vec![] }, &*shader_program);
        shader_instance.bind_texture(&BindPoint::new("sTexture"), &font_texture);
        shader_instance.bind_sampler(&BindPoint::new("sSampler"), &image_sampler);

        Arc::new(Self {
            font_texture,
            context: imgui_context,
            _gfx: gfx.clone(),
            shader_program,
            shader_instance,
            image_sampler,
            render_pass: imgui_render_pass,
        })
    }

    pub fn instantiate_for_surface(&self, surface: &Arc<dyn GfxSurface>) -> Arc<dyn RenderPassInstance> {
        let render_pass_instance = self.render_pass.instantiate(surface, surface.get_extent());

        struct ImGuiRenderPassData {
            shader_program: Arc<dyn ShaderProgram>,
        }
        impl GraphRenderCallback for ImGuiRenderPassData {
            fn draw(&self, command_buffer: &Arc<dyn GfxCommandBuffer>) {
                let io = unsafe { &mut *igGetIO() };
                io.DisplaySize = ImVec2 { x: command_buffer.get_surface().get_extent().x as f32, y: command_buffer.get_surface().get_extent().y as f32 };
                io.DisplayFramebufferScale = ImVec2 { x: 1.0, y: 1.0 };
                io.DeltaTime = 1.0 / 60.0; //@TODO application::get().delta_time();


                unsafe { igNewFrame(); }
                unsafe { igEndFrame(); }
                unsafe { igRender(); }
                let draw_data = unsafe { &*igGetDrawData() };
                let width = draw_data.DisplaySize.x * draw_data.FramebufferScale.x;
                let height = draw_data.DisplaySize.x * draw_data.FramebufferScale.x;
                if width <= 0.0 || height <= 0.0 || draw_data.TotalVtxCount == 0 {
                    return;
                }

                let scale_x = 2.0 / draw_data.DisplaySize.x;
                let scale_y = -2.0 / draw_data.DisplaySize.y;

                #[repr(C, align(4))]
                pub struct ImGuiPushConstants {
                    scale_x: f32,
                    scale_y: f32,
                    translate_x: f32,
                    translate_y: f32,
                }

                command_buffer.push_constant(
                    &self.shader_program,
                    BufferMemory::from_struct(&ImGuiPushConstants {
                        scale_x,
                        scale_y,
                        translate_x: -1.0 - draw_data.DisplayPos.x * scale_x,
                        translate_y: -1.0 - draw_data.DisplayPos.y * scale_y,
                    }),
                    ShaderStage::Vertex,
                )


                for n in 0..draw_data.CmdListsCount
                {
                    let cmd_list = unsafe { &**draw_data.CmdLists.offset(n as isize) };
                    for cmd_i in 0..cmd_list.CmdBuffer.Size
                    {
                        let pcmd = unsafe{*cmd_list.CmdBuffer[cmd_i as isize]};
                        if pcmd.UserCallback != null_mut() {
                            pcmd.UserCallback(cmd_list, pcmd);
                        }
                        else
                        {
                            // Project scissor/clipping rectangles into framebuffer space
                            ImVec4
                            clip_rect;
                            clip_rect.x = (pcmd -> ClipRect.x - clip_off.x) *clip_scale.x;
                            clip_rect.y = (pcmd -> ClipRect.y - clip_off.y) *clip_scale.y;
                            clip_rect.z = (pcmd -> ClipRect.z - clip_off.x) *clip_scale.x;
                            clip_rect.w = (pcmd -> ClipRect.w - clip_off.y) *clip_scale.y;

                            if (clip_rect.x < fb_width && clip_rect.y < fb_height && clip_rect.z >= 0.0f && clip_rect.w >= 0.0f)
                            {
                                // Negative offsets are illegal for vkCmdSetScissor
                                if (clip_rect.x < 0.0f)
                                clip_rect.x = 0.0f;
                                if (clip_rect.y < 0.0f)
                                clip_rect.y = 0.0f;

                                // Apply scissor/clipping rectangle
                                command_buffer -> set_scissor(gfx::Scissor {
                                .offset_x = static_cast<int32_t>(clip_rect.x),
                                .offset_y = static_cast<int32_t>(clip_rect.y),
                                .width    = static_cast<uint32_t>(clip_rect.z - clip_rect.x),
                                .height   = static_cast<uint32_t>(clip_rect.w - clip_rect.y),
                            });

                                // Bind descriptor set with font or user texture
                                if (pcmd -> TextureId && false)
                                imgui_material_instance -> bind_texture("test", nullptr); // TODO handle textures

                                command_buffer -> draw_mesh(mesh, imgui_material_instance.get(), pcmd->IdxOffset + global_idx_offset, pcmd->VtxOffset + global_vtx_offset, pcmd->ElemCount);
                            }
                        }
                    }
                    global_idx_offset += cmd_list -> IdxBuffer.Size;
                    global_vtx_offset += cmd_list -> VtxBuffer.Size;
                }
            }
        }
        render_pass_instance.on_render(Box::new(ImGuiRenderPassData { shader_program: self.shader_program.clone() }));
        render_pass_instance
    }
}