

pub mod backend {
    use std::sync::Arc;
    use backend_vulkan::{GfxVulkan, InstanceCreateInfos};
    use gfx::{GfxRef};
    use gfx::surface::GfxSurface;
    use plateform::window::Window;
    use core::engine::Engine;

    #[cfg(windows)]
    use backend_vulkan_win32::vk_surface_win32::VkSurfaceWin32;
    #[cfg(windows)]
    use plateform_win32::PlatformWin32;

    #[cfg(unix)]
    use backend_vulkan_wayland::vk_surface_wayland::VkSurfaceWayland;
    #[cfg(unix)]
    use plateform_wayland::PlatformWayland;

    pub fn create_engine_vulkan() -> Arc<Engine> {
        #[cfg(windows)]
        return Engine::new(PlatformWin32::new(), GfxVulkan::new(InstanceCreateInfos {
            enable_validation_layers: true,
            required_extensions: vec![("VK_KHR_win32_surface".to_string(), true)],
            ..InstanceCreateInfos::default()
        }));
        #[cfg(unix)]
        return Engine::new(PlatformWayland::new(), GfxVulkan::new(
            InstanceCreateInfos {
                enable_validation_layers: true,
                required_extensions: vec![("VK_KHR_wayland_surface".to_string(), true)],
                ..InstanceCreateInfos::default()
            }
        ));
    }
    
    pub fn create_surface_vulkan(gfx: &GfxRef, window: &Arc<dyn Window>) -> Arc<dyn GfxSurface> {
        #[cfg(windows)]
        return VkSurfaceWin32::new(gfx, format!("{}_surface", window.get_title()), window, 3);
        #[cfg(unix)]
        return VkSurfaceWayland::new(gfx, format!("{}_surface", window.get_title()), window, 3);
    }
}