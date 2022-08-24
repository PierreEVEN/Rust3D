use backend_vulkan::GfxVulkan;
use backend_vulkan_win32::vk_surface_win32::VkSurfaceWin32;
use gfx::render_pass::FrameGraph;
use maths::rect2d::Rect2D;
use maths::vec4::Vec4F32;
use plateform::Platform;
use plateform::window::{PlatformEvent, WindowCreateInfos, WindowFlagBits, WindowFlags};
use plateform_win32::PlatformWin32;

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
    let main_framegraph = FrameGraph::from_surface(&gfx_backend, &main_window_surface, Vec4F32::new(1.0, 0.0, 0.0, 1.0));
    let secondary_framegraph = FrameGraph::from_surface(&gfx_backend, &secondary_window_surface, Vec4F32::new(0.0, 1.0, 0.0, 1.0));

    // Game loop
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

        match main_framegraph.begin() {
            Ok(_) => {
                // Rendering
                main_framegraph.submit();
            }
            Err(_) => {}
        };


        match secondary_framegraph.begin() {
            Ok(_) => {
                // Rendering
                secondary_framegraph.submit();
            }
            Err(_) => {}
        };
    }
}



