use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use backend_vulkan::GfxVulkan;
use backend_vulkan_win32::vk_surface_win32::VkSurfaceWin32;
use core::asset::*;
use core::asset_manager::*;
use core::base_assets::material_asset::*;
use gfx::buffer::BufferMemory;
use gfx::command_buffer::GfxCommandBuffer;
use gfx::image_sampler::SamplerCreateInfos;
use gfx::render_pass::{FrameGraph, GraphRenderCallback, RenderPassAttachment, RenderPassCreateInfos};
use gfx::shader::{PassID, ShaderStage};
use gfx::shader_instance::{BindPoint, ShaderInstance, ShaderInstanceCreateInfos};
use gfx::types::{ClearValues, PixelFormat};
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

    // Create graphics
    let gfx_backend = GfxVulkan::new();
    gfx_backend.set_physical_device(gfx_backend.find_best_suitable_physical_device().expect("there is no suitable GPU available"));

    // Bind graphic surface onto current window
    let main_window_surface = VkSurfaceWin32::new(&gfx_backend, main_window.clone(), 3);
    // Create framegraph
    let main_framegraph = FrameGraph::from_surface(&gfx_backend, &main_window_surface, Vec4F32::new(1.0, 0.0, 0.0, 1.0));

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
    let image_catinou = match read_image_from_file(&gfx_backend, Path::new("data/textures/cat_stretching.png")) {
        Ok(image2) => { image2 }
        Err(error) => { panic!("failed to create image : {}", error.to_string()) }
    };

    // Create sampler
    let sampler = gfx_backend.create_image_sampler(SamplerCreateInfos {});

    // Create material instance
    let shader_instance = gfx_backend.create_shader_instance(ShaderInstanceCreateInfos {
        bindings: demo_material.get_program(&PassID::new("surface_pass")).unwrap().get_bindings()
    }, &*demo_material.get_program(&PassID::new("surface_pass")).unwrap());
    shader_instance.bind_sampler(&BindPoint::new("ui_sampler"), &sampler);

    let shader_2_instance = gfx_backend.create_shader_instance(ShaderInstanceCreateInfos {
        bindings: demo_material.get_program(&PassID::new("surface_pass")).unwrap().get_bindings()
    }, &*demo_material.get_program(&PassID::new("surface_pass")).unwrap());
    shader_2_instance.bind_texture(&BindPoint::new("ui_result"), &image_catinou);
    shader_2_instance.bind_sampler(&BindPoint::new("ui_sampler"), &sampler);

    struct TestGraph {
        start: Instant,
        demo_material: Arc<MaterialAsset>,
        shader_instance: Arc<dyn ShaderInstance>,
        time_pc_data: RwLock<TestPc>,
    }
    #[repr(C, align(4))]
    struct TestPc {
        time: f32,
    }

    impl GraphRenderCallback for TestGraph {
        fn draw(&self, command_buffer: &Arc<dyn GfxCommandBuffer>) {
            self.time_pc_data.write().unwrap().time = self.start.elapsed().as_millis() as f32 / 1000.0;
            match self.demo_material.get_program(&command_buffer.get_pass_id()) {
                None => {
                    panic!("failed to find compatible permutation [{}]", command_buffer.get_pass_id());
                }
                Some(program) => {
                    command_buffer.bind_program(&program);
                    command_buffer.bind_shader_instance(&self.shader_instance);
                    let pc_data = self.time_pc_data.read().unwrap();
                    command_buffer.push_constant(&program, BufferMemory::from_struct(&*pc_data), ShaderStage::Fragment);
                    command_buffer.draw_procedural(4, 0, 1, 0);
                }
            };
        }
    }

    main_framegraph.main_pass().on_render(Box::new(TestGraph { start: Instant::now(), demo_material: demo_material.clone(), shader_instance: shader_instance.clone(), time_pc_data: RwLock::new(TestPc { time: 0.5 }) }));

    let deferred_combine_pass = gfx_backend.create_render_pass(RenderPassCreateInfos {
        name: "deferred_combine".to_string(),
        color_attachments: vec![RenderPassAttachment {
            name: "color".to_string(),
            clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
            image_format: PixelFormat::R8G8B8A8_UNORM,
        }],
        depth_attachment: None,
        is_present_pass: false,
    });

    let def_combine = deferred_combine_pass.instantiate(&main_window_surface, main_window_surface.get_extent());
    def_combine.on_render(Box::new(TestGraph { start: Instant::now(), demo_material, shader_instance: shader_2_instance, time_pc_data: RwLock::new(TestPc { time: 0.5 }) }));
    main_framegraph.main_pass().attach(def_combine.clone());

    shader_instance.bind_texture(&BindPoint::new("ui_result"), &def_combine.get_images()[0]);

    // Game loop
    'game_loop: loop {
        // handle events
        while let Some(message) = platform.poll_event() {
            match message {
                PlatformEvent::WindowClosed(_) => {
                    break 'game_loop;
                }
                PlatformEvent::WindowResized(_window, _width, _height) => {}
            }
        }

        match main_framegraph.begin() {
            Ok(_) => { main_framegraph.submit(); }
            Err(_) => {}
        };
    }
}