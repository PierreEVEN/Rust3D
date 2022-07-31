use std::ffi::c_void;
use std::ptr::null;
use ash::extensions::khr;
use ash::vk;
use ash::vk::{HINSTANCE, SurfaceKHR, Win32SurfaceCreateInfoKHR};
use windows::Win32::Foundation::HWND;
use backend_vulkan::{g_vulkan, G_VULKAN, gfx_object, GfxVulkan};
use plateform_win32::PlatformWin32;
use plateform_win32::window::WindowWin32;


pub struct VkSurfaceWin32 {
    surface: SurfaceKHR
}

impl VkSurfaceWin32 {
    pub fn new(gfx: &GfxVulkan, platform: &PlatformWin32, window: &WindowWin32) -> VkSurfaceWin32 {
        
        let ci_surface = Win32SurfaceCreateInfoKHR {
            flags: Default::default(),
            hinstance: null(),
            hwnd: window.hwnd.0 as *const c_void,
            ..Default::default()
        };
        
        let entry = g_vulkan!();
        
        let surface_fn = khr::Win32Surface::new(entry, &gfx_object!(gfx.instance).instance);
        let surface = unsafe { surface_fn.create_win32_surface(&ci_surface, None) }.expect("failed to create surface");
        
        Self {
            surface
        }
    }
} 