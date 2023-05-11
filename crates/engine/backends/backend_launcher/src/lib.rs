

pub mod backend {
    use backend_vulkan::GfxVulkan;

    #[cfg(windows)]
    use plateform::Platform;
    #[cfg(windows)]
    use plateform_win32::PlatformWin32;

    pub fn spawn_platform() -> Box<dyn Platform> {
        #[cfg(windows)]
        {
            Box::<PlatformWin32>::default()
        }
    }

    pub fn spawn_gfx() -> Box<dyn gfx::GfxInterface> {
        #[cfg(windows)]
        {
            Box::<GfxVulkan>::default()
        }
    }
    
    /*
    pub fn create_surface_vulkan(gfx: &GfxRef, window: &Arc<dyn Window>) -> Arc<dyn GfxSurface> {
        #[cfg(windows)]
        VkSurfaceWin32::new_ptr(gfx, format!("{}_surface", window.get_title()), window, 3)
    }
     */
}