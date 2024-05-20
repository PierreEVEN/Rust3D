use std::ptr::null;
use std::sync::{Arc, RwLock};

use ash::extensions::khr;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk;
use ash::vk::{HINSTANCE, HWND};
use gpu_allocator::vulkan;
use raw_window_handle::RawWindowHandle;

use backend_vulkan::{GfxVulkan, vk_check};
use backend_vulkan::renderer::vk_render_pass_instance::{RbSemaphore, VkRenderPassInstance};
use backend_vulkan::vk_device::VkQueue;
use backend_vulkan::vk_image::VkImage;
use backend_vulkan::vk_types::GfxPixelFormat;
use core::gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use core::gfx::image::{GfxImage, GfxImageUsageFlags, ImageParams, ImageType};
use core::gfx::renderer::render_pass::RenderPassInstance;
use core::gfx::surface::{Frame, GfxSurface, SurfaceAcquireResult};
use logger::fatal;
use plateform::window::{Window, WindowStatus};
use shader_base::types::BackgroundColor;

pub struct VkSurfaceWin32 {
    pub surface: vk::SurfaceKHR,
    pub swapchain: RwLock<Option<vk::SwapchainKHR>>,
    image_acquire_semaphore: Arc<GfxResource<vk::Semaphore>>,
    surface_format: vk::SurfaceFormatKHR,
    _surface_loader: Surface,
    _swapchain_loader: Swapchain,
    image_count: u8,
    window: Arc<dyn Window>,
    surface_image: RwLock<Option<Arc<dyn GfxImage>>>,
    present_queue: Option<Arc<VkQueue>>,
}

struct RbSurfaceImage {
    images: Vec<vk::Image>,
}

impl GfxImageBuilder<(vk::Image, Arc<vulkan::Allocation>)> for RbSurfaceImage {
    fn build(&self, _swapchain_ref: &Frame) -> (vk::Image, Arc<vulkan::Allocation>) {
        (
            self.images[_swapchain_ref.image_id() as usize],
            Arc::new(vulkan::Allocation::default()),
        )
    }
}

impl GfxSurface for VkSurfaceWin32 {
    fn create_or_recreate(&self) {
        vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .device_wait_idle()
        });

        let surface_capabilities = match unsafe {
            self._surface_loader
                .get_physical_device_surface_capabilities(
                    GfxVulkan::get().physical_device_vk.handle,
                    self.surface,
                )
        } {
            Ok(surface_capabilities) => surface_capabilities,
            Err(_) => {
                return;
            }
        };

        if surface_capabilities.current_extent.width == 0
            || surface_capabilities.current_extent.height == 0
        {
            return;
        }

        let present_modes = vk_check!(unsafe {
            self._surface_loader
                .get_physical_device_surface_present_modes(
                    GfxVulkan::get().physical_device_vk.handle,
                    self.surface,
                )
        });

        let mut composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;
        for alpha_flag in &[
            vk::CompositeAlphaFlagsKHR::OPAQUE,
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
            vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
            vk::CompositeAlphaFlagsKHR::INHERIT,
        ] {
            if surface_capabilities
                .supported_composite_alpha
                .contains(*alpha_flag)
            {
                composite_alpha = *alpha_flag;
            }
        }
        let mut present_mode = vk::PresentModeKHR::FIFO;
        for mode in &present_modes {
            if mode.as_raw() == vk::PresentModeKHR::MAILBOX.as_raw() {
                present_mode = *mode;
                break;
            }
        }

        let transform_flags = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };

        let ci_swapchain = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.surface)
            .min_image_count(self.image_count as u32)
            .image_format(self.surface_format.format)
            .image_color_space(self.surface_format.color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(transform_flags)
            .composite_alpha(composite_alpha)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(match *self.swapchain.read().unwrap() {
                None => Default::default(),
                Some(old) => old,
            })
            .build();

        let swapchain =
            vk_check!(unsafe { self._swapchain_loader.create_swapchain(&ci_swapchain, None) });
        GfxVulkan::get().set_vk_object_name(
            swapchain,
            format!("swapchain\t\t: {}", self.window.get_title()).as_str(),
        );

        let mut swapchain_ref = self.swapchain.write().unwrap();
        *swapchain_ref = Some(swapchain);

        let images = vk_check!(unsafe { self._swapchain_loader.get_swapchain_images(swapchain) });

        for (i, image) in images.iter().enumerate() {
            GfxVulkan::get().set_vk_object_name(
                *image,
                format!(
                    "swapchain image\t: surface('{}')@[0:{}]",
                    self.window.get_title(),
                    i
                )
                    .as_str(),
            );
        }

        let mut image = self.surface_image.write().unwrap();
        match &*image {
            None => {
                *image = Some(VkImage::from_existing_images(
                    format!("surface('{}')", self.window.get_title()),
                    GfxResource::new(RbSurfaceImage { images }),
                    ImageParams {
                        pixel_format: *GfxPixelFormat::from(self.surface_format.format),
                        image_type: ImageType::Texture2d(
                            surface_capabilities.current_extent.width,
                            surface_capabilities.current_extent.height,
                        ),
                        read_only: false,
                        mip_levels: Some(1),
                        usage: GfxImageUsageFlags::empty(),
                        background_color: BackgroundColor::None,
                    },
                ));
            }
            Some(image) => {
                image.cast::<VkImage>().update_source(GfxResource::new(RbSurfaceImage { images }), ImageType::Texture2d(surface_capabilities.current_extent.width, surface_capabilities.current_extent.height));
            }
        }
    }

    fn get_owning_window(&self) -> &Arc<dyn Window> {
        &self.window
    }

    fn get_surface_texture(&self) -> Arc<dyn GfxImage> {
        self.surface_image.read().unwrap().as_ref().unwrap().clone()
    }

    fn acquire(
        &self,
        render_pass: &dyn RenderPassInstance,
        global_frame: &Frame
    ) -> Result<(Frame), SurfaceAcquireResult> {

        let geometry = self.window.get_geometry();

        if let WindowStatus::Minimized = self.window.get_status() {
            return Err(SurfaceAcquireResult::Failed("window is minimized".to_string()));
        }

        if geometry.width() == 0 || geometry.height() == 0 {
            return Err(SurfaceAcquireResult::Failed(
                "invalid resolution".to_string(),
            ));
        }

        let current_image_acquire_semaphore = self.image_acquire_semaphore.get(global_frame);
        logger::warning!("B) set image_acquire semaphore {global_frame}");
        let swapchain = self.swapchain.read().unwrap();
        let (acquired_image, _acquired_image) = match unsafe {
            self._swapchain_loader.acquire_next_image(
                swapchain.unwrap(),
                u64::MAX,
                current_image_acquire_semaphore,
                vk::Fence::default(),
            )
        } {
            Ok(result) => result,
            Err(acquire_error) => {
                return Err(match acquire_error {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.create_or_recreate();
                        SurfaceAcquireResult::Resized
                    }
                    _ => SurfaceAcquireResult::Failed("failed to acquire image".to_string()),
                });
            }
        };
        logger::warning!("C) ACQUIRED {global_frame} -> {acquired_image}");
        render_pass.cast::<VkRenderPassInstance>().init_present_pass(self.image_acquire_semaphore.clone(), (global_frame.clone(), Frame::new(acquired_image as u8)));
        Ok(Frame::new(acquired_image as u8))
    }
    fn submit(
        &self,
        render_pass: &dyn RenderPassInstance,
        frame: &Frame,
    ) -> Result<(), SurfaceAcquireResult> {
        // Submit
        let render_pass = render_pass.cast::<VkRenderPassInstance>();

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[render_pass.render_finished_semaphore.get(&frame)])
            .swapchains(&[self.swapchain.read().unwrap().unwrap()])
            .image_indices(&[render_pass.get_acquired_frame(frame).image_id() as u32])
            .build();

        match &self.present_queue {
            None => Err(SurfaceAcquireResult::Failed("no present queue".to_string())),
            Some(queue) => match queue.present(&self._swapchain_loader, present_info) {
                Ok(_) => Ok(()),
                Err(present_error) => Err(match present_error {
                    vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => {
                        self.create_or_recreate();
                        SurfaceAcquireResult::Resized
                    }
                    _ => SurfaceAcquireResult::Failed(present_error.to_string()),
                }),
            },
        }
    }
}


impl VkSurfaceWin32 {
    pub fn new(
        name: String,
        window: &Arc<dyn Window>,
        image_count: u32,
    ) -> Self {
        let handle = match window.get_handle() {
            RawWindowHandle::Win32(handle) => handle,
            _ => {
                fatal!("invalid window handle");
            }
        };

        let ci_surface = vk::Win32SurfaceCreateInfoKHR::builder()
            .hinstance(match handle.hinstance {
                None => { null() }
                Some(hi) => { hi.get() as HINSTANCE }
            })
            .hwnd(handle.hwnd.get() as HWND)
            .build();

        let surface_fn = khr::Win32Surface::new(GfxVulkan::get().entry(), unsafe {
            GfxVulkan::get()
                .instance
                .assume_init_ref()
                .handle
                .assume_init_ref()
        });
        let surface = unsafe { surface_fn.create_win32_surface(&ci_surface, None) }
            .expect("failed to create surface");
        let surface_loader = Surface::new(GfxVulkan::get().entry(), unsafe {
            GfxVulkan::get()
                .instance
                .assume_init_ref()
                .handle
                .assume_init_ref()
        });

        let swapchain_loader = unsafe {
            Swapchain::new(
                GfxVulkan::get()
                    .instance
                    .assume_init_ref()
                    .handle
                    .assume_init_ref(),
                &GfxVulkan::get().device.assume_init_ref().handle,
            )
        };

        let mut image_acquire_semaphore = Vec::new();
        for _ in 0..image_count {
            image_acquire_semaphore.push(vk_check!(unsafe {
                GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .handle
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
            }))
        }

        let surface_formats = vk_check!(unsafe {
            surface_loader.get_physical_device_surface_formats(
                GfxVulkan::get().physical_device_vk.handle,
                surface,
            )
        });
        let mut surface_format: vk::SurfaceFormatKHR = Default::default();
        if surface_formats.len() == 1 && surface_formats[0].format == vk::Format::UNDEFINED {
            surface_format.format = vk::Format::B8G8R8A8_UNORM;
            surface_format.color_space = surface_formats[0].color_space;
        } else {
            let mut found_b8g8r8a8_unorm = false;
            for format in &surface_formats {
                if format.format == vk::Format::B8G8R8A8_UNORM {
                    surface_format.format = format.format;
                    surface_format.color_space = format.color_space;
                    found_b8g8r8a8_unorm = true;
                    break;
                }
            }

            if !found_b8g8r8a8_unorm {
                surface_format.format = surface_formats[0].format;
                surface_format.color_space = surface_formats[0].color_space;
            }
        }

        let mut present_queue = None;

        for (index, _) in (0_u32..).zip(
            unsafe {
                GfxVulkan::get()
                    .instance
                    .assume_init_ref()
                    .handle
                    .assume_init_ref()
                    .get_physical_device_queue_family_properties(
                        GfxVulkan::get().physical_device_vk.handle,
                    )
            }
                .into_iter(),
        ) {
            if vk_check!(unsafe {
                surface_loader.get_physical_device_surface_support(
                    GfxVulkan::get().physical_device_vk.handle,
                    index,
                    surface,
                )
            }) {
                let queue = unsafe {
                    GfxVulkan::get()
                        .device
                        .assume_init_ref()
                        .handle
                        .get_device_queue(index, 0)
                };
                unsafe {
                    present_queue = Some(VkQueue::new(
                        &GfxVulkan::get().device.assume_init_ref().handle,
                        queue,
                        vk::QueueFlags::empty(),
                        index,
                    ));
                }
                break;
            }
        }

        let surface = Self {
            surface,
            swapchain: Default::default(),
            _surface_loader: surface_loader,
            surface_format,
            _swapchain_loader: swapchain_loader,
            image_count: image_count as u8,
            window: window.clone(),
            present_queue,
            image_acquire_semaphore: Arc::new(GfxResource::new(RbSemaphore { name })),
            surface_image: RwLock::default(),
        };

        surface.create_or_recreate();
        logger::info!("Created vulkan surface for win32 platform");
        surface
    }
}
