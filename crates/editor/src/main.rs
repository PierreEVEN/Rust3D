use std::fs;
use std::path::Path;
use std::time::Instant;

use backend_launcher::backend;
use core::asset::*;
use core::base_assets::material_asset::*;
use core::engine::Engine;
use gfx::buffer::BufferMemory;
use gfx::image_sampler::SamplerCreateInfos;
use gfx::render_pass::{FrameGraph, RenderPassAttachment, RenderPassCreateInfos};
use gfx::shader::{PassID, ShaderStage};
use gfx::shader_instance::BindPoint;
use gfx::types::{ClearValues, PixelFormat};
use imgui::ImGUiContext;
use maths::rect2d::Rect2D;
use maths::vec2::Vec2u32;
use maths::vec4::Vec4F32;
use plateform::input_system::{InputAction, InputMapping, KeyboardKey};
use plateform::window::{PlatformEvent, WindowCreateInfos, WindowFlagBits, WindowFlags};
use third_party_io::image::read_image_from_file;

mod gfx_demo;

#[repr(C, align(4))]
struct TestPc {
    time: f32,
}


#[macro_export]
macro_rules! test {
        ( toto ) => {
            println!("coucou");
        };
        ( tata ) => {
            
        };
    }

fn main() {
    
    let _tokens = quote::quote! {
        let a : u32;
        println!("a");
    };
    
    test!(toto);
    
    // We use a win32 backend with a vulkan renderer
    let engine = backend::create_engine_vulkan();

    // Create default inputs
    let input_manager = engine.platform.input_manager();
    input_manager.new_action("MoveForward", InputAction::new().map(InputMapping::Keyboard(KeyboardKey::KeyZ)));
    input_manager.new_action("MoveRight", InputAction::new().map(InputMapping::Keyboard(KeyboardKey::KeyD)));
    input_manager.new_action("MoveLeft", InputAction::new().map(InputMapping::Keyboard(KeyboardKey::KeyQ)));
    input_manager.new_action("MoveBackward", InputAction::new().map(InputMapping::Keyboard(KeyboardKey::KeyS)));

    // Create main window, render surface and framegraph
    let main_window = engine.platform.create_window(WindowCreateInfos {
        name: "Engine - 0.1.0".to_string(),
        geometry: Rect2D::rect(300, 400, 800, 600),
        window_flags: WindowFlags::from_flag(WindowFlagBits::Resizable),
        background_alpha: 255,
    }).expect("failed to create main window");
    main_window.show();
    let main_window_surface = backend::create_surface_vulkan(&engine.gfx, &main_window);

    // Create ImGui context
    let imgui_context = ImGUiContext::new(&engine.gfx);

    // Create render pass and pass instances
    let g_buffer_pass = engine.gfx.create_render_pass(format!("gbuffer"), RenderPassCreateInfos {
        pass_id: PassID::new("deferred_combine"),
        color_attachments: vec![RenderPassAttachment {
            name: "color".to_string(),
            clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
            image_format: PixelFormat::R8G8B8A8_UNORM,
        }],
        depth_attachment: None,
        is_present_pass: false,
    });
    let def_combine = g_buffer_pass.instantiate(&main_window_surface, main_window_surface.get_extent());
    let imgui_pass = imgui_context.instantiate_for_surface(&main_window_surface);

    // Create framegraph
    let main_framegraph = FrameGraph::from_surface(&engine.gfx, &main_window_surface, Vec4F32::new(1.0, 0.0, 0.0, 1.0));
    main_framegraph.main_pass().attach(def_combine.clone());
    main_framegraph.main_pass().attach(imgui_pass.clone());

    // Create material
    let demo_material = MaterialAsset::new(&engine.asset_manager);
    demo_material.meta_data().set_save_path(Path::new("data/demo_shader"));
    demo_material.meta_data().set_name("demo shader".to_string());
    demo_material.set_shader_code(Path::new("data/shaders/resolve.shb"), fs::read_to_string("data/shaders/resolve.shb").expect("failed to read shader_file"));

    // Create images
    let background_image = read_image_from_file(&engine.gfx, Path::new("data/textures/cat_stretching.png")).expect("failed to create image");

    // Create sampler
    let generic_image_sampler = engine.gfx.create_image_sampler(format!("bg_image"),SamplerCreateInfos {});

    // Create material instance
    let surface_combine_shader = demo_material.get_program(&PassID::new("surface_pass")).unwrap().instantiate();
    surface_combine_shader.bind_texture(&BindPoint::new("ui_result"), &imgui_pass.get_images()[0]);
    surface_combine_shader.bind_texture(&BindPoint::new("scene_result"), &def_combine.get_images()[0]);
    surface_combine_shader.bind_sampler(&BindPoint::new("global_sampler"), &generic_image_sampler);

    let background_shader = demo_material.get_program(&PassID::new("deferred_combine")).unwrap().instantiate();
    background_shader.bind_texture(&BindPoint::new("bg_texture"), &background_image);
    background_shader.bind_sampler(&BindPoint::new("global_sampler"), &generic_image_sampler);

    {
        let start = Instant::now();
        let mut time_pc_data = TestPc { time: 0.0 };
        let surface_shader_instance = surface_combine_shader.clone();
        let demo_material = demo_material.clone();
        main_framegraph.main_pass().on_render(Box::new(move |command_buffer| {
            match demo_material.get_program(&command_buffer.get_pass_id()) {
                None => { panic!("failed to find compatible permutation [{}]", command_buffer.get_pass_id()); }
                Some(program) => {
                    time_pc_data.time = start.elapsed().as_millis() as f32 / 1000.0;
                    command_buffer.bind_program(&program);
                    command_buffer.bind_shader_instance(&surface_shader_instance);
                    command_buffer.push_constant(&program, BufferMemory::from_struct(&time_pc_data), ShaderStage::Fragment);
                    command_buffer.draw_procedural(4, 0, 1, 0);
                }
            };
        }));
    }

    {
        let start = Instant::now();
        let mut time_pc_data = TestPc { time: 0.0 };
        let shader_2_instance = background_shader.clone();
        def_combine.on_render(Box::new(move |command_buffer| {
            match demo_material.get_program(&command_buffer.get_pass_id()) {
                None => { panic!("failed to find compatible permutation [{}]", command_buffer.get_pass_id()); }
                Some(program) => {
                    time_pc_data.time = start.elapsed().as_millis() as f32 / 1000.0;
                    command_buffer.bind_program(&program);
                    command_buffer.bind_shader_instance(&shader_2_instance);
                    command_buffer.push_constant(&program, BufferMemory::from_struct(&time_pc_data), ShaderStage::Fragment);
                    command_buffer.draw_procedural(4, 0, 1, 0);
                }
            };
        }));
    }

    main_window.bind_event(PlatformEvent::WindowClosed, Box::new(|_| {
        Engine::get().shutdown();
    }));

    main_window.bind_event(PlatformEvent::WindowResized(0, 0), Box::new(move |event| {
        match event {
            PlatformEvent::WindowResized(width, height) => {
                imgui_pass.resize(Vec2u32::new(*width, *height));
                surface_combine_shader.bind_texture(&BindPoint::new("ui_result"), &imgui_pass.get_images()[0]);
                surface_combine_shader.bind_texture(&BindPoint::new("scene_result"), &def_combine.get_images()[0]);
            }
            _ => {}
        }
    }));

    // Game loop
    while engine.run() {
        engine.platform.poll_events();
        match main_framegraph.begin() {
            Ok(_) => { main_framegraph.submit(); }
            Err(_) => {}
        };
    }
}