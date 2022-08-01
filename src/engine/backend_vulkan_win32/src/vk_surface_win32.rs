use std::ptr::null;

use ash::extensions::khr;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::{Bool32, CompositeAlphaFlagsKHR, Format, ImageUsageFlags, PresentModeKHR, SharingMode, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR, SurfaceTransformFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR, Win32SurfaceCreateInfoKHR};
use raw_window_handle::RawWindowHandle;

use backend_vulkan::{g_vulkan, G_VULKAN, gfx_object, GfxVulkan, vk_check};
use backend_vulkan::types::{VkExtent2D};
use gfx::surface::{GfxSurface, SurfaceCreateInfos};
use maths::vec2::Vec2u32;

pub struct VkSurfaceWin32 {
    pub surface: SurfaceKHR,
    pub swapchain: Option<SwapchainKHR>,
    surface_formats: Vec<SurfaceFormatKHR>,
    surface_capabilities: SurfaceCapabilitiesKHR,
    present_modes: Vec<PresentModeKHR>,
    _surface_loader: Surface,
    _swapchain_loader: Swapchain,
}

impl GfxSurface for VkSurfaceWin32 {
    fn create_or_recreate(&mut self, create_infos: SurfaceCreateInfos) {
        let mut composite_alpha = CompositeAlphaFlagsKHR::OPAQUE;
        for alpha_flag in vec![CompositeAlphaFlagsKHR::OPAQUE, CompositeAlphaFlagsKHR::PRE_MULTIPLIED, CompositeAlphaFlagsKHR::POST_MULTIPLIED, CompositeAlphaFlagsKHR::INHERIT] {
            if self.surface_capabilities.supported_composite_alpha.contains(alpha_flag) {
                composite_alpha = alpha_flag;
            }
        }
        let transform_flags = if self.surface_capabilities.supported_transforms.contains(SurfaceTransformFlagsKHR::IDENTITY) { SurfaceTransformFlagsKHR::IDENTITY } else { self.surface_capabilities.current_transform };
        let mut present_mode = PresentModeKHR::FIFO;
        for mode in &self.present_modes {
            if mode.as_raw() == PresentModeKHR::MAILBOX.as_raw() {
                present_mode = *mode;
                break;
            }
        }

        let mut surface_format: SurfaceFormatKHR = Default::default();
        if self.surface_formats.len() == 1 && self.surface_formats[0].format == Format::UNDEFINED
        {
            surface_format.format = Format::B8G8R8A8_UNORM;
            surface_format.color_space = self.surface_formats[0].color_space;
        } else {
            let mut found_b8g8r8a8_unorm = false;
            for format in &self.surface_formats
            {
                if format.format == Format::B8G8R8A8_UNORM
                {
                    surface_format.format = format.format;
                    surface_format.color_space = format.color_space;
                    found_b8g8r8a8_unorm = true;
                    break;
                }
            }

            if !found_b8g8r8a8_unorm
            {
                surface_format.format = self.surface_formats[0].format;
                surface_format.color_space = self.surface_formats[0].color_space;
            }
        }

        let ci_swapchain = SwapchainCreateInfoKHR {
            surface: self.surface,
            min_image_count: create_infos.image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: *VkExtent2D::from(create_infos.extent),
            image_array_layers: 1,
            image_usage: ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: null(),
            pre_transform: transform_flags,
            composite_alpha,
            present_mode,
            clipped: true as Bool32,
            old_swapchain: match self.swapchain {
                None => { Default::default() }
                Some(old) => { old }
            },
            ..Default::default()
        };

        let swapchain = vk_check!(unsafe { self._swapchain_loader.create_swapchain(&ci_swapchain, None) });

        self.swapchain = Some(swapchain);
    }
}

impl VkSurfaceWin32 {
    pub fn new(gfx: &GfxVulkan, window: &dyn plateform::window::Window) -> VkSurfaceWin32 {
        let handle = match window.get_handle() {
            RawWindowHandle::Win32(handle) => { handle }
            _ => { panic!("invalid window handle"); }
        };

        let ci_surface = Win32SurfaceCreateInfoKHR {
            flags: Default::default(),
            hinstance: handle.hinstance,
            hwnd: handle.hwnd,
            ..Default::default()
        };

        let surface_fn = khr::Win32Surface::new(g_vulkan!(), &gfx_object!(gfx.instance).instance);
        let surface = unsafe { surface_fn.create_win32_surface(&ci_surface, None) }.expect("failed to create surface");
        let surface_loader = Surface::new(g_vulkan!(), &gfx_object!(gfx.instance).instance);

        let surface_formats = vk_check!(unsafe { surface_loader.get_physical_device_surface_formats(gfx_object!(gfx.physical_device_vk).device, surface) });
        let surface_capabilities = vk_check!(unsafe { surface_loader.get_physical_device_surface_capabilities(gfx_object!(gfx.physical_device_vk).device, surface) });
        let present_modes = vk_check!(unsafe { surface_loader.get_physical_device_surface_present_modes(gfx_object!(gfx.physical_device_vk).device, surface) });


        let swapchain_loader = Swapchain::new(&gfx_object!(gfx.instance).instance, &gfx_object!(gfx.device).device);

        let mut surface = Self {
            surface,
            swapchain: None,
            _surface_loader: surface_loader,
            surface_formats,
            surface_capabilities,
            present_modes,
            _swapchain_loader: swapchain_loader,
        };

        surface.create_or_recreate(SurfaceCreateInfos {
            image_count: 3,
            extent: Vec2u32::new(window.get_geometry().width() as u32, window.get_geometry().height() as u32),
        });

        surface
    }
} 