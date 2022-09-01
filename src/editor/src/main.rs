use std::fs;
use std::path::Path;

use backend_vulkan::GfxVulkan;
use backend_vulkan_win32::vk_surface_win32::VkSurfaceWin32;
use core::asset::*;
use core::asset_manager::*;
use core::base_assets::material_asset::*;
use gfx::image_sampler::SamplerCreateInfos;
use gfx::render_pass::FrameGraph;
use gfx::shader::PassID;
use gfx::shader_instance::{BindPoint, ShaderInstanceCreateInfos};
use maths::rect2d::Rect2D;
use maths::vec4::Vec4F32;
use plateform::Platform;
use plateform::window::{PlatformEvent, WindowCreateInfos, WindowFlagBits, WindowFlags};
use plateform_win32::PlatformWin32;
use third_party_io::image::read_image_from_file;

mod gfx_demo;

fn main() {
    // We use a win32 backend
    #[cfg(any(target_os = "windows"))]
        let platform = PlatformWin32::new();

    // Create main window
    let main_window = platform.create_window(WindowCreateInfos {
        name: "Engine - 0.1.0".to_string(),
        geometry: Rect2D::rect(300, 400, 800, 600),
        window_flags: WindowFlags::from_flag(WindowFlagBits::Resizable),
        background_alpha: 255,
    }).unwrap();
    main_window.show();

    // Secondary window
    let secondary_window = platform.create_window(WindowCreateInfos {
        name: "Engine - 0.1.0 - Secondary window".to_string(),
        geometry: Rect2D::rect(350, 450, 600, 440),
        window_flags: WindowFlags::from_flag(WindowFlagBits::Resizable),
        background_alpha: 255,
    }).unwrap();
    secondary_window.show();

    // Create graphics
    let gfx_backend = GfxVulkan::new();
    gfx_backend.set_physical_device(gfx_backend.find_best_suitable_physical_device().expect("there is no suitable GPU available"));

    // Bind graphic surface onto current window
    let main_window_surface = VkSurfaceWin32::new(&gfx_backend, main_window.clone(), 3);
    let secondary_window_surface = VkSurfaceWin32::new(&gfx_backend, secondary_window.clone(), 3);

    // Create framegraph
    let mut main_framegraph = Some(FrameGraph::from_surface(&gfx_backend, &main_window_surface, Vec4F32::new(1.0, 0.0, 0.0, 1.0)));
    let mut secondary_framegraph = Some(FrameGraph::from_surface(&gfx_backend, &secondary_window_surface, Vec4F32::new(0.0, 1.0, 0.0, 1.0)));

    // Create asset manager
    let asset_manager = AssetManager::new(&gfx_backend);

    // Create material
    let demo_material = MaterialAsset::new(&asset_manager);
    demo_material.meta_data().set_save_path(Path::new("data/demo_shader"));
    demo_material.meta_data().set_name("demo shader".to_string());
    demo_material.set_shader_code(Path::new("data/shaders/resolve.shb"), match fs::read_to_string("data/shaders/resolve.shb") {
        Ok(file_data) => { file_data }
        Err(_) => { panic!("failed to read shader_file") }
    });

    // Create images
    let image = match read_image_from_file(&gfx_backend, Path::new("data/textures/raw.jpg")) {
        Ok(image) => { image }
        Err(error) => { panic!("failed to create image : {}", error.to_string()) }
    };

    // Create sampler
    let sampler = gfx_backend.create_image_sampler(SamplerCreateInfos {});

    // Create material instance
    let shader_instance = gfx_backend.create_shader_instance(ShaderInstanceCreateInfos {
        bindings: demo_material.get_program(&PassID::new("surface_pass")).unwrap().get_bindings()
    }, &*demo_material.get_program(&PassID::new("surface_pass")).unwrap());
    shader_instance.bind_texture(&BindPoint::new("ui_result"), &image);
    shader_instance.bind_sampler(&BindPoint::new("ui_sampler"), &sampler);

    main_framegraph.unwrap().main_pass().on_render(|command_buffer| {
        match demo_material.get_program(&command_buffer.get_pass_id()) {
            None => {
                panic!("failed to find compatible permutation [{}]", command_buffer.get_pass_id());
            }
            Some(program) => {
                command_buffer.bind_program(&main_window_surface.get_current_ref(), &program);
                command_buffer.bind_shader_instance(&main_window_surface.get_current_ref(), &shader_instance);
                command_buffer.draw_procedural(&main_window_surface.get_current_ref(), 10, 0, 1, 0);
            }
        };
    });

    // Game loop
    'game_loop: loop {
        // handle events
        while let Some(message) = platform.poll_event() {
            match message {
                PlatformEvent::WindowClosed(window) => {
                    if window.get_handle() == main_window.get_handle() { main_framegraph = None; }
                    if window.get_handle() == secondary_window.get_handle() { secondary_framegraph = None; }
                    if main_framegraph.is_none() && secondary_framegraph.is_none() { break 'game_loop; }
                }
                PlatformEvent::WindowResized(_window, _width, _height) => {}
            }
        }

        match &main_framegraph {
            None => {}
            Some(main_framegraph) => {
                match main_framegraph.begin() {
                    Ok(_) => { main_framegraph.submit(); }
                    Err(_) => {}
                };
            }
        }

        match &secondary_framegraph {
            None => {}
            Some(secondary_framegraph) => {
                match secondary_framegraph.begin() {
                    Ok(_) => { secondary_framegraph.submit(); }
                    Err(_) => {}
                };
            }
        }
    }
}