use std::os::raw::c_char;
use std::ptr::null_mut;
use std::sync::Arc;

use gfx::GfxRef;
use gfx::image::{GfxImage, GfxImageUsageFlags, ImageCreateInfos, ImageParams, ImageUsage};
use gfx::image::ImageType::Texture2d;
use gfx::types::PixelFormat;
use imgui_bindings::{igCreateContext, igGetIO, igGetMainViewport, igGetStyle, igStyleColorsDark, ImFontAtlas_GetTexDataAsRGBA32, ImGuiBackendFlags__ImGuiBackendFlags_HasMouseCursors, ImGuiBackendFlags__ImGuiBackendFlags_HasSetMousePos, ImGuiBackendFlags__ImGuiBackendFlags_PlatformHasViewports, ImGuiConfigFlags__ImGuiConfigFlags_DockingEnable, ImGuiConfigFlags__ImGuiConfigFlags_NavEnableGamepad, ImGuiConfigFlags__ImGuiConfigFlags_NavEnableKeyboard, ImGuiConfigFlags__ImGuiConfigFlags_ViewportsEnable, ImGuiContext, ImTextureID};

pub struct ImGUiContext {
    pub font_texture: Arc<dyn GfxImage>,
    pub context: *mut ImGuiContext,
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

        Arc::new(Self {
            font_texture,
            context: imgui_context,
        })
    }
}