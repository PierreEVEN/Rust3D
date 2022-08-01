use backend_vulkan::{GfxVulkan};
use backend_vulkan_win32::vk_surface_win32::{VkSurfaceWin32};
use gfx::GfxInterface;
use maths::rect2d::Rect2D;
use plateform::Platform;
use plateform::window::{PlatformEvent, WindowCreateInfos, WindowFlagBits, WindowFlags};
use plateform_win32::PlatformWin32;

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