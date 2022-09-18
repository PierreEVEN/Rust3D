

pub mod backend {
    use std::sync::Arc;
    use backend_vulkan::GfxVulkan;
    use gfx::{GfxRef};
    use gfx::surface::GfxSurface;
    use plateform::window::Window;
    use core::engine::Engine;

    #[cfg(windows)]
    use backend_vulkan_win32::vk_surface_win32::VkSurfaceWin32;
    #[cfg(windows)]
    use plateform_win32::PlatformWin32;

    pub fn create_engine_vulkan() -> Arc<Engine> {
        #[cfg(windows)]
        Engine::new(PlatformWin32::new(), GfxVulkan::new())
    }
    
    pub fn create_surface_vulkan(gfx: &GfxRef, window: &Arc<dyn Window>) -> Arc<dyn GfxSurface> {
        #[cfg(windows)]
        VkSurfaceWin32::new(gfx, window, 3)
    }
}