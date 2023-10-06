pub mod backend {
    use std::sync::{Weak};
    use backend_vulkan::GfxVulkan;
    use backend_vulkan_win32::vk_surface_win32::VkSurfaceWin32;
    use gfx::surface::GfxSurface;

    #[cfg(windows)]
    use plateform::Platform;
    #[cfg(windows)]
    use plateform_win32::PlatformWin32;

    pub fn spawn_platform() -> Box<dyn Platform> {
        #[cfg(windows)]
        {
            PlatformWin32::new()
        }
    }

    pub fn spawn_gfx() -> Box<dyn gfx::GfxInterface> {
        #[cfg(windows)]
        {
            Box::<GfxVulkan>::default()
        }
    }

    pub fn spawn_surface(window: &Weak<dyn plateform::window::Window>) -> Box<dyn GfxSurface> {
        #[cfg(windows)]
        {
            Box::new(VkSurfaceWin32::new(format!("{}_surface", window.upgrade().unwrap().get_title()), &window.upgrade().unwrap(), 3))
        }
    }
}
