use std::path::Path;
use backend_vulkan::{GfxVulkan};
use backend_vulkan_win32::vk_surface_win32::{VkSurfaceWin32};
use gfx::buffer::{BufferAccess, BufferCreateInfo, BufferType, BufferUsage};
use gfx::GfxInterface;
use gfx::shader::Shader;
use maths::rect2d::Rect2D;
use plateform::Platform;
use plateform::window::{PlatformEvent, WindowCreateInfos, WindowFlagBits, WindowFlags};
use plateform_win32::PlatformWin32;
use shader_compiler::backends::backend_shaderc::{BackendShaderC, ShaderCIncluder};
use shader_compiler::{CompilationResult, CompilerBackend};
use shader_compiler::parser::{Parser, ShaderChunk};
use shader_compiler::types::{InterstageData, ShaderErrorResult, ShaderLanguage, ShaderStage};

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
    main_window.lock().unwrap().show();

    // Create graphics
    let mut gfx_backend = GfxVulkan::new();
    gfx_backend.set_physical_device(gfx_backend.find_best_suitable_physical_device().expect("there is no suitable GPU available"));

    // Bind graphic surface onto current window
    let mut _main_window_surface = VkSurfaceWin32::new(&gfx_backend, &*main_window.lock().unwrap());

    let mut _test_buffer = gfx_backend.create_buffer(&BufferCreateInfo {
        buffer_type: BufferType::Immutable,
        usage: BufferUsage::IndexData,
        access: BufferAccess::Default,
        size: 2048,
        alignment: 16,
        memory_type_bits: 1,
    });

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
            binding_index: 0
        };
        
        let vertex_code = match parse_result.program_data.get_data(&pass, &ShaderStage::Vertex) {
            Ok(code) => {code}
            Err(error) => {panic!("failed to get vertex shader code : \n{}", error.to_string())}
        };

        let sprv = match shader_compiler.compile_to_spirv(vertex_code, ShaderLanguage::HLSL, ShaderStage::Vertex, interstage) {
            Ok(sprv) => {
                println!("compilation succeeded");
                sprv
            }
            Err(error) => { panic!("shader compilation error : \n{}", error.to_string()) }
        };
    }
    
    let shader = Shader::new()
    
    'game_loop: loop {
        // handle events
        while let Some(message) = platform.poll_event() {
            match message {
                PlatformEvent::WindowClosed(_window) => {
                    break 'game_loop;
                }
                PlatformEvent::WindowResized(_window, _width, _height) => {}
            }
        }

        // Game loop
        gfx_backend.begin_frame();
        gfx_backend.end_frame();
    }
}