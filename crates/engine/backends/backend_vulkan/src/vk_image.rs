use std::sync::{Arc, RwLock};

use ash::vk;
use ash::vk::Handle;
use gpu_allocator::vulkan;

use gfx::buffer::{BufferAccess, BufferCreateInfo, BufferType, BufferUsage};
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::image::{
    GfxImage, GfxImageUsageFlags, ImageCreateInfos, ImageParams, ImageType, ImageUsage,
};
use gfx::surface::GfxImageID;
use gfx::types::PixelFormat;
use gfx::Gfx;

use crate::vk_buffer::VkBufferAccess;
use crate::vk_command_buffer::{
    begin_command_buffer, create_command_buffer, end_command_buffer, submit_command_buffer,
};
use crate::vk_types::VkPixelFormat;
use crate::{vk_check, GfxVulkan, VkBuffer};

type CombinedImageData = (vk::Image, Arc<vulkan::Allocation>);

pub struct VkImage {
    pub image: RwLock<Arc<GfxResource<CombinedImageData>>>,
    pub view: GfxResource<(vk::ImageView, vk::DescriptorImageInfo)>,
    pub image_params: ImageParams,
    pub image_layout: RwLock<vk::ImageLayout>,
    image_type: RwLock<ImageType>,
    is_from_existing_images: bool,
    name: String,
}

impl GfxImage for VkImage {
    fn get_type(&self) -> ImageType {
        *self.image_type.read().unwrap()
    }

    fn get_format(&self) -> PixelFormat {
        self.image_params.pixel_format
    }

    fn get_data(&self) -> &[u8] {
        todo!()
    }

    fn set_data(&self, data: &[u8]) {
        if data.len() != self.get_data_size() as usize {
            logger::fatal!(
                "invalid image memory length : {} (expected {})",
                data.len(),
                self.get_data_size()
            );
        }
        if self.image_params.read_only {
            // Create data transfer buffer
            let transfer_buffer = Gfx::get().create_buffer(
                format!("image[{}]::buffer", self.name),
                &BufferCreateInfo {
                    buffer_type: BufferType::Static,
                    usage: BufferUsage::TransferMemory,
                    access: BufferAccess::CpuToGpu,
                    size: data.len() as u32,
                },
            );

            unsafe {
                // Copy image data to transfer buffer
                transfer_buffer.set_data(&GfxImageID::null(), 0, data);

                // Transfer commands
                let command_buffer = create_command_buffer(format!("{}_transfer_", self.name));
                begin_command_buffer(command_buffer, true);
                self.set_image_layout(
                    &GfxImageID::null(),
                    command_buffer,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
                // GPU copy command
                GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .handle
                    .cmd_copy_buffer_to_image(
                        command_buffer,
                        transfer_buffer
                            .cast::<VkBuffer>()
                            .get_handle(&GfxImageID::null()),
                        self.image.read().unwrap().get_static().0,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[vk::BufferImageCopy::builder()
                            .buffer_offset(0)
                            .buffer_row_length(0)
                            .buffer_image_height(0)
                            .image_subresource(
                                vk::ImageSubresourceLayers::builder()
                                    .aspect_mask(
                                        if self.image_params.pixel_format.is_depth_format() {
                                            vk::ImageAspectFlags::DEPTH
                                        } else {
                                            vk::ImageAspectFlags::COLOR
                                        },
                                    )
                                    .mip_level(0)
                                    .base_array_layer(0)
                                    .layer_count(1)
                                    .build(),
                            )
                            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                            .image_extent(vk::Extent3D {
                                width: self.get_type().dimensions().0,
                                height: self.get_type().dimensions().1,
                                depth: self.get_type().dimensions().2,
                            })
                            .build()],
                    );
                self.set_image_layout(
                    &GfxImageID::null(),
                    command_buffer,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
                end_command_buffer(command_buffer);
                submit_command_buffer(command_buffer, vk::QueueFlags::TRANSFER);
            }
            unsafe {
                match GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .get_queue(vk::QueueFlags::TRANSFER)
                {
                    Ok(queue) => {
                        queue.wait();
                    }
                    Err(_) => {
                        logger::fatal!("failed to find queue");
                    }
                }
            }
        } else {
            logger::fatal!("Applying modification to non-static image is not allowed yet");
        }
    }

    fn get_data_size(&self) -> u32 {
        self.image_params.pixel_format.type_size() * self.image_params.image_type.pixel_count()
    }

    fn resize(&self, new_type: ImageType) {
        if self.is_from_existing_images {
            logger::fatal!("image created from existing images cannot be resized");
        }

        if new_type == self.get_type() {
            return;
        }

        self.image.read().unwrap().invalidate(RbImage {
            create_infos: self.image_params,
            type_override: new_type,
            name: self.name.clone(),
        });
        self.view.invalidate(RbImageView {
            create_infos: self.image_params,
            images: self.image.read().unwrap().clone(),
            type_override: new_type,
            name: self.name.clone(),
        });
        *self.image_type.write().unwrap() = new_type;
    }

    fn __static_view_handle(&self) -> u64 {
        self.view.get_static().0.as_raw()
    }
}

pub struct VkImageUsage(vk::ImageUsageFlags);

impl VkImageUsage {
    fn from(usage: GfxImageUsageFlags, is_depth: bool) -> Self {
        let mut flags = vk::ImageUsageFlags::default();
        if usage.contains(ImageUsage::CopySource) {
            flags |= vk::ImageUsageFlags::TRANSFER_SRC
        }
        if usage.contains(ImageUsage::CopyDestination) {
            flags |= vk::ImageUsageFlags::TRANSFER_DST
        }
        if usage.contains(ImageUsage::Sampling) {
            flags |= vk::ImageUsageFlags::SAMPLED
        }
        if usage.contains(ImageUsage::GpuWriteDestination) {
            flags |= if is_depth {
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
            } else {
                vk::ImageUsageFlags::COLOR_ATTACHMENT
            }
        }

        VkImageUsage(flags)
    }
}

pub struct RbImage {
    create_infos: ImageParams,
    type_override: ImageType,
    name: String,
}

impl GfxImageBuilder<CombinedImageData> for RbImage {
    fn build(&self, swapchain_ref: &GfxImageID) -> CombinedImageData {
        // Convert image details
        let (image_type, width, height, depth) = match self.type_override {
            ImageType::Texture1d(x) => (vk::ImageType::TYPE_1D, x, 1, 1),
            ImageType::Texture2d(x, y) => (vk::ImageType::TYPE_2D, x, y, 1),
            ImageType::Texture3d(x, y, z) => (vk::ImageType::TYPE_3D, x, y, z),
            ImageType::Texture1dArray(x) => (vk::ImageType::TYPE_1D, x, 1, 1),
            ImageType::Texture2dArray(x, y) => (vk::ImageType::TYPE_2D, x, y, 1),
            ImageType::TextureCube(x, y) => (vk::ImageType::TYPE_2D, x, y, 1),
        };
        let create_infos = vk::ImageCreateInfo::builder()
            .image_type(image_type)
            .format(*VkPixelFormat::from(&self.create_infos.pixel_format))
            .extent(vk::Extent3D {
                width,
                height,
                depth,
            })
            .mip_levels(self.create_infos.get_mip_levels() as u32)
            .array_layers(self.create_infos.array_layers())
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(
                VkImageUsage::from(
                    self.create_infos.usage | ImageUsage::CopyDestination,
                    self.create_infos.pixel_format.is_depth_format(),
                )
                .0,
            )
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        // Create image
        let image = vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .create_image(&create_infos, None)
        });

        GfxVulkan::get().set_vk_object_name(
            image,
            format!("texture image\t\t: {}@{}", self.name, swapchain_ref).as_str(),
        );

        // Allocate image memory
        let allocation = unsafe { GfxVulkan::get().device.assume_init_ref() }
            .allocator
            .write()
            .unwrap()
            .allocate(&vulkan::AllocationCreateDesc {
                name: "buffer allocation",
                requirements: unsafe {
                    GfxVulkan::get()
                        .device
                        .assume_init_ref()
                        .handle
                        .get_image_memory_requirements(image)
                },
                location: *VkBufferAccess::from(BufferAccess::GpuOnly),
                linear: true,
                allocation_scheme: vulkan::AllocationScheme::DedicatedImage(image),
            });

        if allocation.is_err() {
            logger::fatal!("failed to allocate image memory");
        }

        let allocation = allocation.unwrap();

        unsafe {
            GfxVulkan::get().set_vk_object_name(
                allocation.memory(),
                format!("texture memory\t\t: {}@{}", self.name, swapchain_ref).as_str(),
            );
        }

        vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .bind_image_memory(image, allocation.memory(), allocation.offset())
        });
        (image, Arc::new(allocation))
    }
}

pub struct RbImageView {
    images: Arc<GfxResource<(vk::Image, Arc<vulkan::Allocation>)>>,
    create_infos: ImageParams,
    type_override: ImageType,
    name: String,
}

impl GfxImageBuilder<(vk::ImageView, vk::DescriptorImageInfo)> for RbImageView {
    fn build(&self, swapchain_ref: &GfxImageID) -> (vk::ImageView, vk::DescriptorImageInfo) {
        let view_type = match self.type_override {
            ImageType::Texture1d(_) => vk::ImageViewType::TYPE_1D,
            ImageType::Texture2d(_, _) => vk::ImageViewType::TYPE_2D,
            ImageType::Texture3d(_, _, _) => vk::ImageViewType::TYPE_3D,
            ImageType::Texture1dArray(_) => vk::ImageViewType::TYPE_1D_ARRAY,
            ImageType::Texture2dArray(_, _) => vk::ImageViewType::TYPE_2D_ARRAY,
            ImageType::TextureCube(_, _) => vk::ImageViewType::CUBE,
        };

        if self.create_infos.read_only {
            let view = vk_check!(unsafe {
                GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .handle
                    .create_image_view(
                        &vk::ImageViewCreateInfo::builder()
                            .image(self.images.get_static().0)
                            .view_type(view_type)
                            .format(*VkPixelFormat::from(&self.create_infos.pixel_format))
                            .components(vk::ComponentMapping {
                                r: vk::ComponentSwizzle::R,
                                g: vk::ComponentSwizzle::G,
                                b: vk::ComponentSwizzle::B,
                                a: vk::ComponentSwizzle::A,
                            })
                            .subresource_range(
                                vk::ImageSubresourceRange::builder()
                                    .aspect_mask(
                                        if self.create_infos.pixel_format.is_depth_format() {
                                            vk::ImageAspectFlags::DEPTH
                                        } else {
                                            vk::ImageAspectFlags::COLOR
                                        },
                                    )
                                    .base_mip_level(0)
                                    .level_count(match self.create_infos.mip_levels {
                                        None => 1,
                                        Some(levels) => levels as u32,
                                    })
                                    .base_array_layer(0)
                                    .layer_count(1)
                                    .build(),
                            )
                            .build(),
                        None,
                    )
            });
            GfxVulkan::get().set_vk_object_name(
                view,
                format!("image view\t\t: {}@{}", self.name, swapchain_ref).as_str(),
            );
            (
                view,
                vk::DescriptorImageInfo {
                    sampler: Default::default(),
                    image_view: view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                },
            )
        } else {
            let device = &GfxVulkan::get().device;
            let view = vk_check!(unsafe {
                device.assume_init_ref().handle.create_image_view(
                    &vk::ImageViewCreateInfo::builder()
                        .image(self.images.get(swapchain_ref).0)
                        .view_type(view_type)
                        .format(*VkPixelFormat::from(&self.create_infos.pixel_format))
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(if self.create_infos.pixel_format.is_depth_format() {
                                    vk::ImageAspectFlags::DEPTH
                                } else {
                                    vk::ImageAspectFlags::COLOR
                                })
                                .base_mip_level(0)
                                .level_count(match self.create_infos.mip_levels {
                                    None => 1,
                                    Some(levels) => levels as u32,
                                })
                                .base_array_layer(0)
                                .layer_count(1)
                                .build(),
                        )
                        .build(),
                    None,
                )
            });
            GfxVulkan::get().set_vk_object_name(
                view,
                format!("image view\t\t: {}@{}", self.name, swapchain_ref).as_str(),
            );
            (
                view,
                vk::DescriptorImageInfo {
                    sampler: Default::default(),
                    image_view: view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                },
            )
        }
    }
}

impl VkImage {
    pub fn new_ptr(name: String, create_infos: ImageCreateInfos) -> Arc<dyn GfxImage> {
        let params = create_infos.params;

        let image_views = if params.read_only {
            // Static image
            let images = Arc::new(GfxResource::new_static(RbImage {
                create_infos: params,
                type_override: create_infos.params.image_type,
                name: name.clone(),
            }));
            (
                images.clone(),
                GfxResource::new_static(RbImageView {
                    create_infos: params,
                    images,
                    type_override: create_infos.params.image_type,
                    name: name.clone(),
                }),
            )
        } else {
            // Dynamic image
            let images = Arc::new(GfxResource::new(RbImage {
                create_infos: params,
                type_override: create_infos.params.image_type,
                name: name.clone(),
            }));
            (
                images.clone(),
                GfxResource::new(RbImageView {
                    create_infos: params,
                    images,
                    type_override: create_infos.params.image_type,
                    name: name.clone(),
                }),
            )
        };

        let image = Arc::new(Self {
            view: image_views.1,
            image: RwLock::new(image_views.0),
            image_params: params,
            image_layout: RwLock::new(vk::ImageLayout::UNDEFINED),
            image_type: RwLock::new(params.image_type),
            is_from_existing_images: false,
            name,
        });

        match create_infos.pixels {
            None => {}
            Some(pixels) => image.set_data(pixels.as_slice()),
        }
        image
    }

    pub fn from_existing_images(
        name: String,
        existing_images: GfxResource<CombinedImageData>,
        image_params: ImageParams,
    ) -> Arc<dyn GfxImage> {
        if existing_images.is_static() != image_params.read_only {
            logger::fatal!("trying to create framebuffer from existing images, but images was created as read only")
        }

        let image_usage = image_params;
        let images = Arc::new(existing_images);
        Arc::new(Self {
            image: RwLock::new(images.clone()),
            view: GfxResource::new(RbImageView {
                images,
                create_infos: image_usage,
                type_override: image_usage.image_type,
                name: name.clone(),
            }),
            image_params: image_usage,
            image_layout: RwLock::new(vk::ImageLayout::UNDEFINED),
            image_type: RwLock::new(image_usage.image_type),
            is_from_existing_images: true,
            name,
        })
    }

    fn set_image_layout(
        &self,
        _: &GfxImageID,
        command_buffer: vk::CommandBuffer,
        new_layout: vk::ImageLayout,
    ) {
        if self.image_params.read_only {
            let mut current_layout = self.image_layout.write().unwrap();
            let mut barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(*current_layout)
                .new_layout(new_layout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.image.read().unwrap().get_static().0)
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(if self.image_params.pixel_format.is_depth_format() {
                            vk::ImageAspectFlags::DEPTH
                        } else {
                            vk::ImageAspectFlags::COLOR
                        })
                        .base_mip_level(0)
                        .level_count(match self.image_params.mip_levels {
                            Some(levels) => levels as u32,
                            None => 1,
                        })
                        .base_array_layer(0)
                        .layer_count(match self.image_params.image_type {
                            ImageType::TextureCube(_, _) => 6,
                            _ => 1,
                        })
                        .build(),
                )
                .build();

            let source_destination_stages = if *current_layout == vk::ImageLayout::UNDEFINED
                && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            {
                barrier.src_access_mask = vk::AccessFlags::NONE;
                barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

                (
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                )
            } else if *current_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
                && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            {
                barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
                barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

                (
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                )
            } else {
                logger::fatal!("Unsupported layout transition");
            };

            *current_layout = new_layout;

            let device = &GfxVulkan::get().device;
            unsafe {
                device.assume_init_ref().handle.cmd_pipeline_barrier(
                    command_buffer,
                    source_destination_stages.0,
                    source_destination_stages.1,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );
            }
        } else {
            logger::fatal!("changing image layout on dynamic images is not supported yet");
        }
    }

    pub fn resize_from_existing_images(
        &self,
        new_type: ImageType,
        existing_images: Arc<GfxResource<CombinedImageData>>,
    ) {
        *self.image.write().unwrap() = existing_images;
        self.view.invalidate(RbImageView {
            create_infos: self.image_params,
            images: self.image.read().unwrap().clone(),
            type_override: new_type,
            name: self.name.clone(),
        });
        *self.image_type.write().unwrap() = new_type;
    }
}
