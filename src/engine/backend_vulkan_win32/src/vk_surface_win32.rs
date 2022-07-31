use ash::vk;
use ash::vk::{HINSTANCE, Win32SurfaceCreateInfoKHR};
use backend_vulkan::{g_vulkan, gfx_object, GfxVulkan};
use plateform_win32::PlatformWin32;
use plateform_win32::window::WindowWin32;


pub struct VkSurfaceWin32 {
  
    
}

impl VkSurfaceWin32 {
    pub fn new(gfx: &GfxVulkan, platform: &PlatformWin32, window: &WindowWin32) -> VkSurfaceWin32 {
        
        let ci_surface = Win32SurfaceCreateInfoKHR {
            flags: Default::default(),
            hinstance: HINSTANCE::default(),
            hwnd: window.hwnd as ash::vk::HWND,
            ..Default::default()
        };
        
        let instance = &gfx_object!(gfx.instance).instance;
        
        let surface = unsafe { ash_window::create_surface(g_vulkan!(), , window.hwnd as ash::vk::HWND, None) };
        
        Self {}
    }
} 