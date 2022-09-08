use std::slice;
use std::sync::{Arc, RwLock};

use ash::vk;
use ash::vk::{AccessFlags, BufferImageCopy, CommandBuffer, ComponentMapping, ComponentSwizzle, DependencyFlags, DescriptorImageInfo, Extent3D, Handle, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers, ImageSubresourceRange, ImageTiling, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, Offset3D, PipelineStageFlags, QUEUE_FAMILY_IGNORED, QueueFlags, SampleCountFlags, SharingMode};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc};

use gfx::buffer::{BufferAccess, BufferCreateInfo, BufferType, BufferUsage};
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::image::{GfxImage, GfxImageUsageFlags, ImageCreateInfos, ImageParams, ImageType, ImageUsage};
use gfx::surface::GfxImageID;
use gfx::types::PixelFormat;

use crate::{GfxVulkan, vk_check, VkBuffer};
use crate::vk_buffer::VkBufferAccess;
use crate::vk_command_buffer::{begin_command_buffer, create_command_buffer, end_command_buffer, submit_command_buffer};
use crate::vk_types::VkPixelFormat;

type CombinedImageData = (Image, Arc<Allocation>);

pub struct VkImage {
    gfx: GfxRef,
    pub image: Arc<GfxResource<CombinedImageData>>,
    pub view: GfxResource<(ImageView, DescriptorImageInfo)>,
    pub image_params: ImageParams,
    pub image_layout: RwLock<ImageLayout>,
}

impl GfxImage for VkImage {
    fn get_type(&self) -> ImageType {
        self.image_params.image_type
    }

    fn get_format(&self) -> PixelFormat {
        self.image_params.pixel_format
    }

    fn get_data(&self) ->  &[u8] {
        todo!()
    }

    fn set_data(&self, data: &[u8]) {
        if data.len() != self.get_data_size() as usize {
            panic!("invalid image memory length : {} (expected {})", data.len(), self.get_data_size());
        }
        if self.image_params.read_only {
            // Create data transfer buffer
            let transfer_buffer = self.gfx.create_buffer(&BufferCreateInfo {
                buffer_type: BufferType::Static,
                usage: BufferUsage::TransferMemory,
                access: BufferAccess::CpuToGpu,
                size: data.len() as u32,
            });

            unsafe {
                // Copy image data to transfer buffer      
                transfer_buffer.set_data(&GfxImageID::null(), 0, data);

                // Transfer commands
                let command_buffer = create_command_buffer(&self.gfx);
                begin_command_buffer(&self.gfx, command_buffer, true);
                self.set_image_layout(&GfxImageID::null(), command_buffer, ImageLayout::TRANSFER_DST_OPTIMAL);
                // GPU copy command
                self.gfx.cast::<GfxVulkan>().device.handle.cmd_copy_buffer_to_image(
                    command_buffer,
                    transfer_buffer.cast::<VkBuffer>().get_handle(&GfxImageID::null()),
                    self.image.get_static().0,
                    ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[BufferImageCopy {
                        buffer_offset: 0,
                        buffer_row_length: 0,
                        buffer_image_height: 0,
                        image_subresource: ImageSubresourceLayers {
                            aspect_mask: if self.image_params.pixel_format.is_depth_format() { ImageAspectFlags::DEPTH } else { ImageAspectFlags::COLOR },
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        image_offset: Offset3D { x: 0, y: 0, z: 0 },
                        image_extent: Extent3D { width: self.get_type().dimensions().0, height: self.get_type().dimensions().1, depth: self.get_type().dimensions().2 },
                    }]);
                self.set_image_layout(&GfxImageID::null(), command_buffer, ImageLayout::SHADER_READ_ONLY_OPTIMAL);
                end_command_buffer(&self.gfx, command_buffer);
                submit_command_buffer(&self.gfx, command_buffer, QueueFlags::TRANSFER);
            }
        } else {
            panic!("Applying modification to non-static image is not allowed yet");
        }
    }

    fn get_data_size(&self) -> u32 {
        self.image_params.pixel_format.type_size() * self.image_params.image_type.pixel_count()
    }

    fn __static_view_handle(&self) -> u64 {
        self.view.get_static().0.as_raw()
    }
}

pub struct VkImageUsage(ImageUsageFlags);

impl VkImageUsage {
    fn from(usage: GfxImageUsageFlags, is_depth: bool) -> Self {
        let mut flags = ImageUsageFlags::default();
        if usage.contains(ImageUsage::CopySource) { flags |= ImageUsageFlags::TRANSFER_SRC }
        if usage.contains(ImageUsage::CopyDestination) { flags |= ImageUsageFlags::TRANSFER_DST }
        if usage.contains(ImageUsage::Sampling) { flags |= ImageUsageFlags::SAMPLED }
        if usage.contains(ImageUsage::GpuWriteDestination) {
            flags |= if is_depth { ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT } else { ImageUsageFlags::COLOR_ATTACHMENT }
        }

        VkImageUsage(flags)
    }
}

pub struct RbImage {
    create_infos: ImageParams,
}

impl GfxImageBuilder<CombinedImageData> for RbImage {
    fn build(&self, gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> CombinedImageData {
        // Convert image details
        let (image_type, width, height, depth) = match self.create_infos.image_type {
            ImageType::Texture1d(x) => { (vk::ImageType::TYPE_1D, x, 1, 1) }
            ImageType::Texture2d(x, y) => { (vk::ImageType::TYPE_2D, x, y, 1) }
            ImageType::Texture3d(x, y, z) => { (vk::ImageType::TYPE_3D, x, y, z) }
            ImageType::Texture1dArray(x) => { (vk::ImageType::TYPE_1D, x, 1, 1) }
            ImageType::Texture2dArray(x, y) => { (vk::ImageType::TYPE_2D, x, y, 1) }
            ImageType::TextureCube(x, y) => { (vk::ImageType::TYPE_2D, x, y, 1) }
        };
        let create_infos = ImageCreateInfo::builder()
            .image_type(image_type)
            .format(*VkPixelFormat::from(&self.create_infos.pixel_format))
            .extent(Extent3D { width, height, depth })
            .mip_levels(self.create_infos.get_mip_levels() as u32)
            .array_layers(self.create_infos.array_layers())
            .samples(SampleCountFlags::TYPE_1)
            .tiling(ImageTiling::OPTIMAL)
            .usage(VkImageUsage::from(self.create_infos.usage | ImageUsage::CopyDestination, self.create_infos.pixel_format.is_depth_format()).0)
            .sharing_mode(SharingMode::EXCLUSIVE)
            .build();
        // Create image
        let image = vk_check!(unsafe {gfx.cast::<GfxVulkan>().device.handle.create_image(
            &create_infos,
            None
        )});

        // Allocate image memory
        let allocation = gfx.cast::<GfxVulkan>().device.allocator.write().unwrap().allocate(&AllocationCreateDesc {
            name: "buffer allocation",
            requirements: unsafe { gfx.cast::<GfxVulkan>().device.handle.get_image_memory_requirements(image) },
            location: *VkBufferAccess::from(BufferAccess::GpuOnly),
            linear: true,
        });

        if allocation.is_err() {
            panic!("failed to allocate image memory");
        }

        let allocation = allocation.unwrap();

        vk_check!(unsafe { gfx.cast::<GfxVulkan>().device.handle.bind_image_memory(image, allocation.memory(), allocation.offset())});
        (image, Arc::new(allocation))
    }
}

pub struct RbImageView {
    images: Arc<GfxResource<(Image, Arc<Allocation>)>>,
    create_infos: ImageParams,
}

impl GfxImageBuilder<(ImageView, DescriptorImageInfo)> for RbImageView {
    fn build(&self, gfx: &GfxRef, swapchain_ref: &GfxImageID) -> (ImageView, DescriptorImageInfo) {
        let view_type = match self.create_infos.image_type {
            ImageType::Texture1d(_) => { ImageViewType::TYPE_1D }
            ImageType::Texture2d(_, _) => { ImageViewType::TYPE_2D }
            ImageType::Texture3d(_, _, _) => { ImageViewType::TYPE_3D }
            ImageType::Texture1dArray(_) => { ImageViewType::TYPE_1D_ARRAY }
            ImageType::Texture2dArray(_, _) => { ImageViewType::TYPE_2D_ARRAY }
            ImageType::TextureCube(_, _) => { ImageViewType::CUBE }
        };

        if self.create_infos.read_only {
            let device = &gfx.cast::<GfxVulkan>().device;
            let view = vk_check!(unsafe { 
                (*device).handle.create_image_view(&ImageViewCreateInfo::builder()
                .image(self.images.get_static().0)
                .view_type(view_type)
                .format(*VkPixelFormat::from(&self.create_infos.pixel_format))
                .components(ComponentMapping { r: ComponentSwizzle::R, g: ComponentSwizzle::G, b: ComponentSwizzle::B, a: ComponentSwizzle::A })
                .subresource_range(ImageSubresourceRange::builder()
                    .aspect_mask(if self.create_infos.pixel_format.is_depth_format() { ImageAspectFlags::DEPTH } else { ImageAspectFlags::COLOR })
                    .base_mip_level(0)
                    .level_count(match self.create_infos.mip_levels {
                        None => { 1 }
                        Some(levels) => { levels as u32 }
                    })
                    .base_array_layer( 0)
                    .layer_count(1)
                    .build())
                .build(), 
                None) 
            });
            (view, DescriptorImageInfo {
                sampler: Default::default(),
                image_view: view,
                image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            })
        } else {
            let device = &gfx.cast::<GfxVulkan>().device;
            let view = vk_check!(unsafe { 
                (*device).handle.create_image_view(&ImageViewCreateInfo::builder()
                .image(self.images.get(swapchain_ref).0)
                .view_type(view_type)
                .format(*VkPixelFormat::from(&self.create_infos.pixel_format))
                .components(ComponentMapping { r: ComponentSwizzle::R, g: ComponentSwizzle::G, b: ComponentSwizzle::B, a: ComponentSwizzle::A })
                .subresource_range(ImageSubresourceRange::builder()
                    .aspect_mask(if self.create_infos.pixel_format.is_depth_format() { ImageAspectFlags::DEPTH } else { ImageAspectFlags::COLOR })
                    .base_mip_level(0)
                    .level_count(match self.create_infos.mip_levels {
                        None => { 1 }
                        Some(levels) => { levels as u32 }
                    })
                    .base_array_layer( 0)
                    .layer_count(1)
                    .build())
                .build(), 
                None) 
            });
            (view, DescriptorImageInfo {
                sampler: Default::default(),
                image_view: view,
                image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            })
        }
    }
}

impl VkImage {
    pub fn new(gfx: &GfxRef, create_infos: ImageCreateInfos) -> Arc<dyn GfxImage> {
        let params = create_infos.params;

        let image_views = if params.read_only {
            // Static image
            let images = Arc::new(GfxResource::new_static(gfx, RbImage { create_infos: params }));
            (
                images.clone(),
                GfxResource::new_static(gfx, RbImageView { create_infos: params, images })
            )
        } else {
            // Dynamic image
            let images = Arc::new(GfxResource::new(gfx, RbImage { create_infos: params }));
            (
                images.clone(),
                GfxResource::new(gfx, RbImageView { create_infos: params, images })
            )
        };

        let image = Arc::new(Self {
            gfx: gfx.clone(),
            view: image_views.1,
            image: image_views.0,
            image_params: params,
            image_layout: RwLock::new(ImageLayout::UNDEFINED),
        });

        match create_infos.pixels {
            None => {}
            Some(pixels) => {
                image.set_data(pixels.as_slice())
            }
        }
        image
    }

    pub fn from_existing_images(gfx: &GfxRef, existing_images: GfxResource<CombinedImageData>, image_params: ImageParams) -> Arc<dyn GfxImage> {
        if existing_images.is_static() != image_params.read_only {
            panic!("trying to create framebuffer from existing images, but images was created as read only")
        }

        let images = Arc::new(existing_images);
        Arc::new(VkImage {
            gfx: gfx.clone(),
            image: images.clone(),
            view: GfxResource::new(gfx, RbImageView {
                images,
                create_infos: image_params,
            }),
            image_params,
            image_layout: RwLock::new(ImageLayout::UNDEFINED),
        })
    }

    fn set_image_layout(&self, _: &GfxImageID, command_buffer: CommandBuffer, new_layout: ImageLayout) {
        if self.image_params.read_only {
            let mut current_layout = self.image_layout.write().unwrap();
            let mut barrier = ImageMemoryBarrier::builder()
                .old_layout(*current_layout)
                .new_layout(new_layout)
                .src_queue_family_index(QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
                .image(self.image.get_static().0)
                .subresource_range(ImageSubresourceRange::builder()
                    .aspect_mask(if self.image_params.pixel_format.is_depth_format() { ImageAspectFlags::DEPTH } else { ImageAspectFlags::COLOR })
                    .base_mip_level(0)
                    .level_count(match self.image_params.mip_levels {
                        Some(levels) => { levels as u32 }
                        None => { 1 }
                    })
                    .base_array_layer(0)
                    .layer_count(match self.image_params.image_type {
                        ImageType::TextureCube(_, _) => { 6 }
                        _ => { 1 }
                    })
                    .build())
                .build();

            let source_destination_stages = if *current_layout == ImageLayout::UNDEFINED && new_layout == ImageLayout::TRANSFER_DST_OPTIMAL
            {
                barrier.src_access_mask = AccessFlags::NONE;
                barrier.dst_access_mask = AccessFlags::TRANSFER_WRITE;

                (PipelineStageFlags::TOP_OF_PIPE, PipelineStageFlags::TRANSFER)
            } else if *current_layout == ImageLayout::TRANSFER_DST_OPTIMAL && new_layout == ImageLayout::SHADER_READ_ONLY_OPTIMAL
            {
                barrier.src_access_mask = AccessFlags::TRANSFER_WRITE;
                barrier.dst_access_mask = AccessFlags::SHADER_READ;

                (PipelineStageFlags::TRANSFER, PipelineStageFlags::FRAGMENT_SHADER)
            } else {
                panic!("Unsupported layout transition");
            };

            *current_layout = new_layout;

            let device = &self.gfx.cast::<GfxVulkan>().device;
            unsafe {
                (*device).handle.cmd_pipeline_barrier(
                    command_buffer,
                    source_destination_stages.0,
                    source_destination_stages.1,
                    DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier]);
            }
        } else {
            panic!("changing image layout on dynamic images is not supported yet");
        }
    }
}