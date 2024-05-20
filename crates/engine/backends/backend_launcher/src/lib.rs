pub mod backend {
    use std::sync::Weak;

    use backend_vulkan::GfxVulkan;
    #[cfg(unix)]
    use backend_vulkan_wayland::vk_surface_wayland::VkSurfaceWayland;
    use backend_vulkan_win32::vk_surface_win32::VkSurfaceWin32;
    use core::gfx::surface::GfxSurface;
    #[cfg(windows)]
    use plateform::Platform;
    #[cfg(unix)]
    use plateform_wayland::PlatformWayland;
    #[cfg(windows)]
    use plateform_win32::PlatformWin32;

    pub fn spawn_platform() -> Box<dyn Platform> {
        #[cfg(windows)]
        {
            return PlatformWin32::new();
        }
        #[cfg(unix)]
        {
            return PlatformWayland::new();
        }
    }

    pub fn spawn_gfx() -> Box<dyn core::gfx::GfxInterface> {
        #[cfg(windows)]
        {
            return Box::<GfxVulkan>::default();
        }
        #[cfg(unix)]
        {
            return Box::<GfxVulkan>::new(
                InstanceCreateInfos {
                    enable_validation_layers: true,
                    required_extensions: vec![("VK_KHR_wayland_surface".to_string(), true)],
                    ..InstanceCreateInfos::default()
                }
            );
        }
    }

    pub fn spawn_surface(window: &Weak<dyn plateform::window::Window>) -> std::sync::Arc<dyn GfxSurface> {
        #[cfg(windows)]
        {
            return std::sync::Arc::new(VkSurfaceWin32::new(format!("{}_surface", window.upgrade().unwrap().get_title()), &window.upgrade().unwrap(), 3));
        }
        #[cfg(unix)]
        {
            return  std::sync::Arc::new(VkSurfaceWayland::new(format!("{}_surface", window.upgrade().unwrap().get_title()), &window.upgrade().unwrap(), 3));
        }
    }
}
