use std::ptr::null;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU8, Ordering};

use ash::extensions::khr;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk;
use ash::vk::{Bool32, ComponentMapping, ComponentSwizzle, CompositeAlphaFlagsKHR, Fence, Format, ImageAspectFlags, ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, PresentInfoKHR, PresentModeKHR, Semaphore, SemaphoreCreateInfo, SharingMode, SurfaceFormatKHR, SurfaceKHR, SurfaceTransformFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR, Win32SurfaceCreateInfoKHR};
use raw_window_handle::RawWindowHandle;

use backend_vulkan::{g_vulkan, G_VULKAN, gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check};
use backend_vulkan::vk_render_pass::VkRenderPass;
use backend_vulkan::vk_render_pass_instance::RbSemaphore;
use backend_vulkan::vk_swapchain_resource::{GfxImageBuilder, VkSwapchainResource};
use backend_vulkan::vk_types::{GfxPixelFormat, VkExtent2D};
use gfx::GfxRef;
use gfx::image::GfxImage;
use gfx::render_pass::{RenderPass, RenderPassCreateInfos};
use gfx::surface::{GfxImageID, GfxSurface};
use gfx::types::PixelFormat;
use maths::vec2::Vec2u32;
use plateform::window::Window;

pub struct VkSurfaceWin32 {
    pub surface: SurfaceKHR,
    pub swapchain: RwLock<Option<SwapchainKHR>>,
    image_acquire_semaphore: VkSwapchainResource<Semaphore>,
    surface_format: SurfaceFormatKHR,
    transform_flags: SurfaceTransformFlagsKHR,
    _surface_loader: Surface,
    _swapchain_loader: Swapchain,
    image_count: u8,
    composite_alpha: CompositeAlphaFlagsKHR,
    present_mode: PresentModeKHR,
    current_image: AtomicU8,
    window: Arc<dyn Window>,
    gfx: GfxRef,
    swapchain_image_views: RwLock<Vec<ImageView>>,
}


impl GfxSurface for VkSurfaceWin32 {
    fn create_or_recreate(&self) {
        let dimensions = Vec2u32::new(self.window.get_geometry().width() as u32, self.window.get_geometry().height() as u32);

        let ci_swapchain = SwapchainCreateInfoKHR {
            surface: self.surface,
            min_image_count: self.image_count as u32,
            image_format: self.surface_format.format,
            image_color_space: self.surface_format.color_space,
            image_extent: *VkExtent2D::from(dimensions),
            image_array_layers: 1,
            image_usage: ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: null(),
            pre_transform: self.transform_flags,
            composite_alpha: self.composite_alpha,
            present_mode: self.present_mode,
            clipped: true as Bool32,
            old_swapchain: match *self.swapchain.read().unwrap() {
                None => { Default::default() }
                Some(old) => { old }
            },
            ..Default::default()
        };

        let swapchain = vk_check!(unsafe { self._swapchain_loader.create_swapchain(&ci_swapchain, None) });

        let mut swapchain_ref = self.swapchain.write().unwrap();
        *swapchain_ref = Some(swapchain);

        let images = vk_check!(unsafe { self._swapchain_loader.get_swapchain_images(swapchain) });

        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();

        let mut views = Vec::new();
        for i in 0..self.get_image_count() {
            let ci_view = ImageViewCreateInfo {
                image: images[i as usize],
                view_type: ImageViewType::TYPE_2D,
                format: self.surface_format.format,
                components: ComponentMapping { r: ComponentSwizzle::R, g: ComponentSwizzle::G, b: ComponentSwizzle::B, a: ComponentSwizzle::A },
                subresource_range: ImageSubresourceRange {
                    aspect_mask: ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                    ..ImageSubresourceRange::default()
                },
                ..ImageViewCreateInfo::default()
            };

            unsafe { views.push(vk_check!(gfx_object!(*device).device.create_image_view(&ci_view, None))) }
        }

        let mut view_list = self.swapchain_image_views.write().unwrap();
        *view_list = views;
    }

    fn get_owning_window(&self) -> &Arc<dyn Window> {
        &self.window
    }

    fn get_surface_pixel_format(&self) -> PixelFormat {
        *GfxPixelFormat::from(self.surface_format.format)
    }

    fn get_image_count(&self) -> u8 {
        self.image_count
    }

    fn get_current_ref(&self) -> GfxImageID {
        GfxImageID::new(self.gfx.clone(), self.current_image.load(Ordering::Acquire), 0)
    }

    fn get_images(&self) -> Vec<Arc<dyn GfxImage>> {
        todo!()
    }

    fn create_render_pass(&self, create_infos: RenderPassCreateInfos) -> Arc<dyn RenderPass> {
        VkRenderPass::new(&self.gfx, create_infos)
    }

    fn get_gfx(&self) -> &GfxRef {
        &self.gfx
    }

    fn begin(&self) -> Result<(), String> {
        let geometry = self.window.get_geometry();

        if geometry.width() == 0 || geometry.height() == 0 {
            return Err("invalid resolution".to_string());
        }

        let current_image_acquire_semaphore = self.image_acquire_semaphore.get(&self.get_current_ref());
        let swapchain = self.swapchain.read().unwrap();
        let (image_index, _acquired_image) = match unsafe { self._swapchain_loader.acquire_next_image(swapchain.unwrap(), u64::MAX, current_image_acquire_semaphore, Fence::default()) } {
            Ok(result) => { result }
            Err(acquire_error) => {
                match acquire_error {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.create_or_recreate();
                    }
                    _ => {}
                };
                (0, false)
            }
        };

        self.current_image.store(image_index as u8, Ordering::Release);

        //@TODO : wait for framegraph fences

        Ok(())
    }

    fn submit(&self) {
        let current_image = self.get_current_ref().image_index as u32;
        let _present_info = PresentInfoKHR {
            wait_semaphore_count: 0,
            p_wait_semaphores: null(),
            swapchain_count: 1,
            p_swapchains: &self.swapchain.read().unwrap().unwrap(),
            p_image_indices: &current_image,
            ..PresentInfoKHR::default()
        };
    }
}

impl VkSurfaceWin32 {
    pub fn new(gfx: &GfxRef, window: Arc<dyn Window>, image_count: u32) -> Arc<dyn GfxSurface> {
        let gfx_copy = gfx.clone();
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        let physical_device_vk = gfx_cast_vulkan!(gfx).physical_device_vk.read().unwrap();

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

        let surface_fn = khr::Win32Surface::new(g_vulkan!(), &gfx_object!(gfx_cast_vulkan!(gfx.clone()).instance).instance);
        let surface = unsafe { surface_fn.create_win32_surface(&ci_surface, None) }.expect("failed to create surface");
        let surface_loader = Surface::new(g_vulkan!(), &gfx_object!(gfx_cast_vulkan!(gfx.clone()).instance).instance);

        let surface_formats = vk_check!(unsafe { surface_loader.get_physical_device_surface_formats(gfx_object!(*physical_device_vk).device, surface) });
        let surface_capabilities = vk_check!(unsafe { surface_loader.get_physical_device_surface_capabilities(gfx_object!(*physical_device_vk).device, surface) });
        let present_modes = vk_check!(unsafe { surface_loader.get_physical_device_surface_present_modes(gfx_object!(*physical_device_vk).device, surface) });


        let swapchain_loader = Swapchain::new(&gfx_object!(gfx_cast_vulkan!(gfx.clone()).instance).instance, &gfx_object!(*device).device);

        let mut image_acquire_semaphore = Vec::new();
        for _ in 0..image_count {
            image_acquire_semaphore.push(vk_check!(unsafe { gfx_object!(*device).device.create_semaphore(&SemaphoreCreateInfo::default(), None) }))
        }


        let mut composite_alpha = CompositeAlphaFlagsKHR::OPAQUE;
        for alpha_flag in vec![CompositeAlphaFlagsKHR::OPAQUE, CompositeAlphaFlagsKHR::PRE_MULTIPLIED, CompositeAlphaFlagsKHR::POST_MULTIPLIED, CompositeAlphaFlagsKHR::INHERIT] {
            if surface_capabilities.supported_composite_alpha.contains(alpha_flag) {
                composite_alpha = alpha_flag;
            }
        }
        let transform_flags = if surface_capabilities.supported_transforms.contains(SurfaceTransformFlagsKHR::IDENTITY) { SurfaceTransformFlagsKHR::IDENTITY } else { surface_capabilities.current_transform };
        let mut present_mode = PresentModeKHR::FIFO;
        for mode in &present_modes {
            if mode.as_raw() == PresentModeKHR::MAILBOX.as_raw() {
                present_mode = *mode;
                break;
            }
        }

        let mut surface_format: SurfaceFormatKHR = Default::default();
        if surface_formats.len() == 1 && surface_formats[0].format == Format::UNDEFINED
        {
            surface_format.format = Format::B8G8R8A8_UNORM;
            surface_format.color_space = surface_formats[0].color_space;
        } else {
            let mut found_b8g8r8a8_unorm = false;
            for format in &surface_formats
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
                surface_format.format = surface_formats[0].format;
                surface_format.color_space = surface_formats[0].color_space;
            }
        }

        let surface = Arc::new(Self {
            surface,
            swapchain: Default::default(),
            _surface_loader: surface_loader,
            surface_format,
            present_mode,
            transform_flags,
            composite_alpha,
            _swapchain_loader: swapchain_loader,
            image_count: image_count as u8,
            current_image: AtomicU8::new(0),
            window: window.clone(),
            gfx: gfx_copy,
            image_acquire_semaphore: VkSwapchainResource::new(Box::new(RbSemaphore {})),
            swapchain_image_views: RwLock::default(),
        });

        surface.create_or_recreate();
        surface
    }
} 