use std::slice;
use std::mem::size_of;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::Arc;

use core::engine::Engine;
use gfx::buffer::{BufferMemory, BufferType};
use gfx::Gfx;
use gfx::image::{GfxImage, GfxImageUsageFlags, ImageCreateInfos, ImageParams, ImageUsage};
use gfx::image::ImageType::Texture2d;
use gfx::image_sampler::{ImageSampler, SamplerCreateInfos};
use gfx::material::Material;
use gfx::mesh::{IndexBufferType, Mesh, MeshCreateInfos};
use gfx::renderer::render_node::RenderNode;
use gfx::renderer::renderer_resource::PassResource;
use imgui_bindings::{
    igCreateContext, igEndFrame, igGetDrawData, igGetIO, igGetMainViewport, igGetStyle, igNewFrame,
    igRender, igShowDemoWindow, igStyleColorsDark, ImDrawIdx, ImDrawVert,
    ImFontAtlas_GetTexDataAsRGBA32, ImGuiBackendFlags__ImGuiBackendFlags_HasMouseCursors,
    ImGuiBackendFlags__ImGuiBackendFlags_HasSetMousePos,
    ImGuiBackendFlags__ImGuiBackendFlags_PlatformHasViewports,
    ImGuiConfigFlags__ImGuiConfigFlags_DockingEnable,
    ImGuiConfigFlags__ImGuiConfigFlags_NavEnableGamepad,
    ImGuiConfigFlags__ImGuiConfigFlags_NavEnableKeyboard,
    ImGuiConfigFlags__ImGuiConfigFlags_ViewportsEnable, ImGuiContext, ImTextureID, ImVec2, ImVec4,
};
use maths::vec2::Vec2f32;
use maths::vec4::Vec4F32;
use plateform::input_system::{InputMapping, MouseButton};
use resl::ReslShaderInterface;
use shader_base::{BindPoint, ShaderStage};
use shader_base::types::{BackgroundColor, PixelFormat, Scissors};

pub struct ImGUiContext {
    pub font_texture: Arc<dyn GfxImage>,
    pub material: Arc<Material>,
    pub image_sampler: Arc<dyn ImageSampler>,
    pub context: *mut ImGuiContext,
    pub mesh: Arc<Mesh>,
    pub render_node: Arc<RenderNode>,
}

impl ImGUiContext {
    pub fn new() -> Self {
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
        io.MouseHoveredViewport = 0;

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
        assert_ne!(io.Fonts as usize, 0, "ImGui font is not valid");
        unsafe {
            ImFontAtlas_GetTexDataAsRGBA32(
                io.Fonts,
                &mut pixels,
                &mut width,
                &mut height,
                null_mut(),
            )
        }
        let data_size = width * height * PixelFormat::R8G8B8A8_UNORM.type_size() as i32;

        let font_texture = Gfx::get().create_image(
            "imgui_font".to_string(),
            ImageCreateInfos {
                params: ImageParams {
                    pixel_format: PixelFormat::R8G8B8A8_UNORM,
                    image_type: Texture2d(width as u32, height as u32),
                    read_only: true,
                    mip_levels: None,
                    usage: GfxImageUsageFlags::from_flag(ImageUsage::Sampling),
                    background_color: BackgroundColor::None,
                },
                pixels: Some(unsafe {
                    Vec::from_raw_parts(pixels, data_size as usize, data_size as usize)
                }),
            },
        );
        unsafe {
            (*io.Fonts).TexID = font_texture.__static_view_handle() as ImTextureID;
        }

        let render_node = RenderNode::default()
            .name("imgui_render_pass")
            .add_resource(PassResource {
                name: "color".to_string(),
                clear_value: BackgroundColor::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
                format: PixelFormat::R8G8B8A8_UNORM,
            })
            .add_resource(PassResource {
                name: "depth".to_string(),
                clear_value: BackgroundColor::DepthStencil(Vec2f32::new(1.0, 0.0)),
                format: PixelFormat::D24_UNORM_S8_UINT,
            });

        let material = Material::default();
        material.set_shader(ReslShaderInterface::from(PathBuf::from("data/shaders/imgui_material.shb")));

        let image_sampler = Gfx::get()
            .create_image_sampler("imgui_default_sampler".to_string(), SamplerCreateInfos {});

        material.bind_texture(&BindPoint::new("sTexture"), font_texture.clone());
        material.bind_sampler(&BindPoint::new("sSampler"), image_sampler.clone());

        let mesh = Gfx::get().create_mesh(
            "imgui_dynamic_mesh".to_string(),
            &MeshCreateInfos {
                vertex_structure_size: size_of::<ImDrawVert>() as u32,
                vertex_count: 0,
                index_count: 0,
                buffer_type: BufferType::Immediate,
                index_buffer_type: IndexBufferType::Uint16,
                vertex_data: None,
                index_data: None,
            },
        );

        render_node.add_render_function(move |world, command_buffer| {
            /*
            let frame = command_buffer.get_frame_id();
            
            let io = unsafe { &mut *igGetIO() };
            io.DisplaySize = ImVec2 {
                x: command_buffer.get_surface().get_extent().x as f32,
                y: command_buffer.get_surface().get_extent().y as f32,
            };
            io.DisplayFramebufferScale = ImVec2 { x: 1.0, y: 1.0 };
            io.DeltaTime = Engine::get().delta_second() as f32;
            
            // Update mouse
            let input_manager = Engine::get().platform().input_manager();
            io.MouseDown[0] =
                input_manager.is_input_pressed(InputMapping::MouseButton(MouseButton::Left));
            io.MouseDown[1] =
                input_manager.is_input_pressed(InputMapping::MouseButton(MouseButton::Right));
            io.MouseDown[2] =
                input_manager.is_input_pressed(InputMapping::MouseButton(MouseButton::Middle));
            io.MouseDown[3] =
                input_manager.is_input_pressed(InputMapping::MouseButton(MouseButton::Button1));
            io.MouseDown[4] =
                input_manager.is_input_pressed(InputMapping::MouseButton(MouseButton::Button2));
            io.MouseHoveredViewport = 0;
            io.MousePos = ImVec2 {
                x: input_manager.get_mouse_position().x,
                y: input_manager.get_mouse_position().y,
            };

            unsafe {
                igNewFrame();
            }
            unsafe {
                igShowDemoWindow(null_mut());
            }
            unsafe {
                igEndFrame();
            }
            unsafe {
                igRender();
            }
            let draw_data = unsafe { &*igGetDrawData() };
            let width = draw_data.DisplaySize.x * draw_data.FramebufferScale.x;
            let height = draw_data.DisplaySize.x * draw_data.FramebufferScale.x;
            if width <= 0.0 || height <= 0.0 || draw_data.TotalVtxCount == 0 {
                return;
            }
            /*
             * BUILD VERTEX BUFFERS
             */
            unsafe {
                let mut vertex_start = 0;
                let mut index_start = 0;

                mesh.resize(
                    &frame,
                    draw_data.TotalVtxCount as u32,
                    draw_data.TotalIdxCount as u32,
                );

                for n in 0..draw_data.CmdListsCount {
                    let cmd_list = &**draw_data.CmdLists.offset(n as isize);

                    mesh.set_data(
                        &frame,
                        vertex_start,
                        slice::from_raw_parts(
                            cmd_list.VtxBuffer.Data as *const u8,
                            cmd_list.VtxBuffer.Size as usize * size_of::<ImDrawVert>(),
                        ),
                        index_start,
                        slice::from_raw_parts(
                            cmd_list.IdxBuffer.Data as *const u8,
                            cmd_list.IdxBuffer.Size as usize * size_of::<ImDrawIdx>(),
                        ),
                    );

                    vertex_start += cmd_list.VtxBuffer.Size as u32;
                    index_start += cmd_list.IdxBuffer.Size as u32;
                }
            }

            /*
             * PREPARE MATERIALS
             */
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
                &material.get_program(&command_buffer.get_pass_id()).unwrap(),
                BufferMemory::from_struct(&ImGuiPushConstants {
                    scale_x,
                    scale_y,
                    translate_x: -1.0 - draw_data.DisplayPos.x * scale_x,
                    translate_y: 1.0 - draw_data.DisplayPos.y * scale_y,
                }),
                ShaderStage::Vertex,
            );

            material.bind_texture(&BindPoint::new("sTexture"), font_texture.clone());

            // Will project scissor/clipping rectangles into framebuffer space
            let clip_off = draw_data.DisplayPos; // (0,0) unless using multi-viewports
            let clip_scale = draw_data.FramebufferScale; // (1,1) unless using retina display which are often (2,2)

            // Render command lists
            // (Because we merged all buffers into a single one, we maintain our own offset into them)
            let mut global_idx_offset = 0;
            let mut global_vtx_offset = 0;

            for n in 0..draw_data.CmdListsCount {
                let cmd = unsafe { &**draw_data.CmdLists.offset(n as isize) };
                for cmd_i in 0..cmd.CmdBuffer.Size {
                    let pcmd = unsafe { &*cmd.CmdBuffer.Data.offset(cmd_i as isize) };
                    match pcmd.UserCallback {
                        Some(callback) => unsafe {
                            callback(cmd, pcmd);
                        },
                        None => {
                            // Project scissor/clipping rectangles into framebuffer space
                            let mut clip_rect = ImVec4 {
                                x: (pcmd.ClipRect.x - clip_off.x) * clip_scale.x,
                                y: (pcmd.ClipRect.y - clip_off.y) * clip_scale.y,
                                z: (pcmd.ClipRect.z - clip_off.x) * clip_scale.x,
                                w: (pcmd.ClipRect.w - clip_off.y) * clip_scale.y,
                            };

                            if clip_rect.x < command_buffer.get_surface().get_extent().x as f32
                                && clip_rect.y < command_buffer.get_surface().get_extent().y as f32
                                && clip_rect.z >= 0.0
                                && clip_rect.w >= 0.0
                            {
                                // Negative offsets are illegal for vkCmdSetScissor
                                if clip_rect.x < 0.0 {
                                    clip_rect.x = 0.0;
                                }
                                if clip_rect.y < 0.0 {
                                    clip_rect.y = 0.0;
                                }

                                // Apply scissor/clipping rectangle
                                command_buffer.set_scissor(Scissors {
                                    min_x: clip_rect.x as i32,
                                    min_y: clip_rect.y as i32,
                                    width: (clip_rect.z - clip_rect.x) as u32,
                                    height: (clip_rect.w - clip_rect.y) as u32,
                                });

                                // Bind descriptor set with font or user texture
                                if !pcmd.TextureId.is_null() {
                                    //material.bind_texture(&BindPoint::new("sTexture"), nullptr); // TODO handle textures
                                }

                                material.bind_to(command_buffer);

                                command_buffer.draw_mesh_advanced(
                                    &mesh,
                                    pcmd.IdxOffset + global_idx_offset,
                                    (pcmd.VtxOffset + global_vtx_offset) as i32,
                                    pcmd.ElemCount,
                                    1,
                                    0,
                                );
                            }
                        }
                    }
                }
                global_idx_offset += cmd.IdxBuffer.Size as u32;
                global_vtx_offset += cmd.VtxBuffer.Size as u32;
            }
             */
        });

        logger::info!("initialized imgui context");
        Self {
            font_texture,
            context: imgui_context,
            material: Arc::new(material),
            image_sampler,
            mesh,
            render_node: Arc::new(render_node),
        }
    }
}
