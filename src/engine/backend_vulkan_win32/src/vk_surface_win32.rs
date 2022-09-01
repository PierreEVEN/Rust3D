use std::ptr::null;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU8, Ordering};

use ash::extensions::khr;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk;
use ash::vk::{Bool32, CompositeAlphaFlagsKHR, Extent2D, Fence, Format, Image, ImageUsageFlags, PresentInfoKHR, PresentModeKHR, QueueFlags, Semaphore, SemaphoreCreateInfo, SharingMode, SurfaceFormatKHR, SurfaceKHR, SurfaceTransformFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR, Win32SurfaceCreateInfoKHR};
use gpu_allocator::vulkan::Allocation;
use raw_window_handle::RawWindowHandle;

use backend_vulkan::{g_vulkan, G_VULKAN, gfx_cast_vulkan, GfxVulkan, vk_check};
use backend_vulkan::vk_device::VkQueue;
use backend_vulkan::vk_image::VkImage;
use backend_vulkan::vk_render_pass_instance::{RbSemaphore, VkRenderPassInstance};
use backend_vulkan::vk_types::GfxPixelFormat;
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::image::{GfxImage, ImageParams, ImageType, ImageUsage};
use gfx::render_pass::{RenderPassInstance};
use gfx::surface::{GfxImageID, GfxSurface, SurfaceAcquireResult};
use gfx::types::PixelFormat;
use maths::vec2::Vec2u32;
use plateform::window::Window;

pub struct VkSurfaceWin32 {
    pub surface: SurfaceKHR,
    pub swapchain: RwLock<Option<SwapchainKHR>>,
    image_acquire_semaphore: GfxResource<Semaphore>,
    surface_format: SurfaceFormatKHR,
    _surface_loader: Surface,
    _swapchain_loader: Swapchain,
    image_count: u8,
    current_image: AtomicU8,
    window: Arc<dyn Window>,
    gfx: GfxRef,
    surface_image: RwLock<Option<Arc<dyn GfxImage>>>,
    present_queue: Option<Arc<VkQueue>>,
    extent: RwLock<Extent2D>,
}


struct RbSurfaceImage {
    images: Vec<Image>,
}

impl GfxImageBuilder<(Image, Arc<Allocation>)> for RbSurfaceImage {
    fn build(&self, _gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> (Image, Arc<Allocation>) {
        (self.images[_swapchain_ref.image_index as usize], Arc::new(Allocation::default()))
    }
}

impl GfxSurface for VkSurfaceWin32 {
    fn create_or_recreate(&self) {
        let device = &gfx_cast_vulkan!(self.gfx).device;
        vk_check!(unsafe { (*device).device.device_wait_idle() });

        let physical_device_vk = &gfx_cast_vulkan!(self.gfx).physical_device_vk;
        let surface_capabilities = match unsafe { self._surface_loader.get_physical_device_surface_capabilities((*physical_device_vk).device, self.surface) } {
            Ok(surface_capabilities) => { surface_capabilities }
            Err(_) => {
                return;
            }
        };
        
        if surface_capabilities.current_extent.width <= 0 || surface_capabilities.current_extent.height <= 0 {
            return;
        }

        let present_modes = vk_check!(unsafe { self._surface_loader.get_physical_device_surface_present_modes((*physical_device_vk).device, self.surface) });

        let mut composite_alpha = CompositeAlphaFlagsKHR::OPAQUE;
        for alpha_flag in vec![CompositeAlphaFlagsKHR::OPAQUE, CompositeAlphaFlagsKHR::PRE_MULTIPLIED, CompositeAlphaFlagsKHR::POST_MULTIPLIED, CompositeAlphaFlagsKHR::INHERIT] {
            if surface_capabilities.supported_composite_alpha.contains(alpha_flag) {
                composite_alpha = alpha_flag;
            }
        }
        let mut present_mode = PresentModeKHR::FIFO;
        for mode in &present_modes {
            if mode.as_raw() == PresentModeKHR::MAILBOX.as_raw() {
                present_mode = *mode;
                break;
            }
        }

        let transform_flags = if surface_capabilities.supported_transforms.contains(SurfaceTransformFlagsKHR::IDENTITY) { SurfaceTransformFlagsKHR::IDENTITY } else { surface_capabilities.current_transform };

        let ci_swapchain = SwapchainCreateInfoKHR {
            surface: self.surface,
            min_image_count: self.image_count as u32,
            image_format: self.surface_format.format,
            image_color_space: self.surface_format.color_space,
            image_extent: surface_capabilities.current_extent,
            image_array_layers: 1,
            image_usage: ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: null(),
            pre_transform: transform_flags,
            composite_alpha,
            present_mode,
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

        let mut image = self.surface_image.write().unwrap();
        *image = Some(VkImage::from_existing_images(&self.gfx, GfxResource::new(Box::new(RbSurfaceImage {
            images,
        })), ImageParams {
            pixel_format: *GfxPixelFormat::from(self.surface_format.format),
            image_format: ImageType::Texture2d(surface_capabilities.current_extent.width, surface_capabilities.current_extent.height),
            read_only: true,
            mip_levels: Some(1),
            usage: ImageUsage::Any,
        }));
        
        *self.extent.write().unwrap() = surface_capabilities.current_extent;
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
        GfxImageID::new(&self.gfx, self.current_image.load(Ordering::Acquire), 0)
    }

    fn get_surface_texture(&self) -> Arc<dyn GfxImage> {
        self.surface_image.read().unwrap().as_ref().unwrap().clone()
    }

    fn get_extent(&self) -> Vec2u32 {
        let extent = self.extent.read().unwrap();
        Vec2u32::new(extent.width, extent.height)
    }

    fn get_gfx(&self) -> &GfxRef {
        &self.gfx
    }

    fn acquire(&self, render_pass: &Arc<dyn RenderPassInstance>) -> Result<(), SurfaceAcquireResult> {
        let geometry = self.window.get_geometry();

        if geometry.width() == 0 || geometry.height() == 0 {
            return Err(SurfaceAcquireResult::Failed("invalid resolution".to_string()));
        }

        let current_image_acquire_semaphore = self.image_acquire_semaphore.get(&self.get_current_ref());
        let swapchain = self.swapchain.read().unwrap();
        let (image_index, _acquired_image) = match unsafe { self._swapchain_loader.acquire_next_image(swapchain.unwrap(), u64::MAX, current_image_acquire_semaphore, Fence::default()) } {
            Ok(result) => { result }
            Err(acquire_error) => {
                return Err(match acquire_error {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.create_or_recreate();
                        SurfaceAcquireResult::Resized
                    }
                    _ => {
                        SurfaceAcquireResult::Failed("failed to acquire image".to_string())
                    }
                });
            }
        };
        self.current_image.store(image_index as u8, Ordering::Release);

        let render_pass = (**render_pass).as_any().downcast_ref::<VkRenderPassInstance>().unwrap();
        let mut wait_sem = render_pass.wait_semaphores.write().unwrap();
        *wait_sem = Some(current_image_acquire_semaphore);

        Ok(())
    }

    fn submit(&self, render_pass: &Arc<dyn RenderPassInstance>) -> Result<(), SurfaceAcquireResult> {
        let current_image = self.get_current_ref().image_index as u32;
        let render_pass = (**render_pass).as_any().downcast_ref::<VkRenderPassInstance>().unwrap();

        let _present_info = PresentInfoKHR {
            wait_semaphore_count: 1,
            p_wait_semaphores: &render_pass.render_finished_semaphore.get(&self.get_current_ref()),
            swapchain_count: 1,
            p_swapchains: &self.swapchain.read().unwrap().unwrap(),
            p_image_indices: &current_image,
            ..PresentInfoKHR::default()
        };

        match &self.present_queue {
            None => { Err(SurfaceAcquireResult::Failed("no present queue".to_string())) }
            Some(queue) => {
                return match queue.present(&self._swapchain_loader, _present_info) {
                    Ok(_) => { Ok(()) }
                    Err(present_error) => {
                        Err(match present_error {
                            vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => {
                                self.create_or_recreate();
                                SurfaceAcquireResult::Resized
                            }
                            _ => {
                                SurfaceAcquireResult::Failed(present_error.to_string())
                            }
                        })
                    }
                };
            }
        }
    }
}

impl VkSurfaceWin32 {
    pub fn new(gfx: &GfxRef, window: Arc<dyn Window>, image_count: u32) -> Arc<dyn GfxSurface> {
        let gfx_copy = gfx.clone();
        let device = &gfx_cast_vulkan!(gfx).device;
        let physical_device_vk = &gfx_cast_vulkan!(gfx).physical_device_vk;

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

        let surface_fn = khr::Win32Surface::new(g_vulkan!(), &gfx_cast_vulkan!(gfx.clone()).instance.instance);
        let surface = unsafe { surface_fn.create_win32_surface(&ci_surface, None) }.expect("failed to create surface");
        let surface_loader = Surface::new(g_vulkan!(), &gfx_cast_vulkan!(gfx.clone()).instance.instance);

        let swapchain_loader = Swapchain::new(&gfx_cast_vulkan!(gfx.clone()).instance.instance, &(*device).device);

        let mut image_acquire_semaphore = Vec::new();
        for _ in 0..image_count {
            image_acquire_semaphore.push(vk_check!(unsafe { (*device).device.create_semaphore(&SemaphoreCreateInfo::default(), None) }))
        }


        let surface_formats = vk_check!(unsafe { surface_loader.get_physical_device_surface_formats((*physical_device_vk).device, surface) });
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

        let mut present_queue = None;

        let instance = &gfx_cast_vulkan!(gfx).instance.instance;

        let mut index: u32 = 0;
        for _ in unsafe { instance.get_physical_device_queue_family_properties((*physical_device_vk).device) } {
            if vk_check!(unsafe { surface_loader.get_physical_device_surface_support((*physical_device_vk).device, index, surface) }) {
                let queue = unsafe { (*device).device.get_device_queue(index, 0) };
                present_queue = Some(VkQueue::new(&(*device).device, queue, QueueFlags::empty(), index, &gfx));
                break;
            }
            index += 1;
        }

        let surface = Arc::new(Self {
            surface,
            swapchain: Default::default(),
            _surface_loader: surface_loader,
            surface_format,
            _swapchain_loader: swapchain_loader,
            image_count: image_count as u8,
            current_image: AtomicU8::new(0),
            window: window.clone(),
            gfx: gfx_copy,
            present_queue,
            image_acquire_semaphore: GfxResource::new(Box::new(RbSemaphore {})),
            surface_image: RwLock::default(),
            extent: RwLock::new(Extent2D { width: 0, height: 0 }),
        });

        surface.create_or_recreate();
        surface
    }
} 