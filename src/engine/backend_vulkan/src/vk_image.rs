use std::sync::{Arc, RwLock};

use ash::vk;
use ash::vk::{AccessFlags, BufferImageCopy, CommandBuffer, ComponentMapping, ComponentSwizzle, DependencyFlags, DeviceSize, Extent3D, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers, ImageSubresourceRange, ImageTiling, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, Offset3D, PipelineStageFlags, QUEUE_FAMILY_IGNORED, QueueFlags, SampleCountFlags, SharingMode};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc};

use gfx::buffer::{BufferAccess, BufferCreateInfo, BufferType, BufferUsage};
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::image::{GfxImage, GfxImageUsageFlags, ImageCreateInfos, ImageParams, ImageType, ImageUsage};
use gfx::surface::GfxImageID;
use gfx::types::PixelFormat;

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check, VkBuffer};
use crate::vk_buffer::VkBufferAccess;
use crate::vk_command_buffer::{begin_command_buffer, create_command_buffer, end_command_buffer, submit_command_buffer};
use crate::vk_types::VkPixelFormat;

pub struct VkImage {
    gfx: GfxRef,
    pub image: Arc<GfxResource<(Image, Arc<Allocation>)>>,
    pub view: GfxResource<ImageView>,
    pub image_params: ImageParams,
    pub image_layout: RwLock<ImageLayout>,
}

impl GfxImage for VkImage {
    fn get_width(&self) -> u32 {
        match self.image_params.image_format {
            ImageType::Texture1d(x) => { x }
            ImageType::Texture2d(x, _) => { x }
            ImageType::Texture3d(x, _, _) => { x }
            ImageType::Texture1dArray(x) => { x }
            ImageType::Texture2dArray(x, _) => { x }
            ImageType::TextureCube(x, _) => { x }
        }
    }

    fn get_height(&self) -> u32 {
        match self.image_params.image_format {
            ImageType::Texture1d(_) => { 1 }
            ImageType::Texture2d(_, y) => { y }
            ImageType::Texture3d(_, y, _) => { y }
            ImageType::Texture1dArray(_) => { 1 }
            ImageType::Texture2dArray(_, y) => { y }
            ImageType::TextureCube(_, y) => { y }
        }
    }

    fn get_depth(&self) -> u32 {
        match self.image_params.image_format {
            ImageType::Texture1d(_) => { 1 }
            ImageType::Texture2d(_, _) => { 1 }
            ImageType::Texture3d(_, _, z) => { z }
            ImageType::Texture1dArray(_) => { 1 }
            ImageType::Texture2dArray(_, _) => { 1 }
            ImageType::TextureCube(_, _) => { 1 }
        }
    }

    fn get_format(&self) -> PixelFormat {
        self.image_params.pixel_format
    }

    fn get_data(&self) -> Vec<u8> {
        todo!()
    }

    fn set_data(&self, data: Vec<u8>) {
        if !self.image_params.read_only {
            panic!("dynamic image type not supported yet");
        }
        if data.len() != self.get_data_size() as usize {
            panic!("wrong texture data size ; {} expected {}", data.len(), self.get_data_size());
        }

        let transfer_buffer = self.gfx.create_buffer(&BufferCreateInfo {
            buffer_type: BufferType::Static,
            usage: BufferUsage::TransferMemory,
            access: BufferAccess::CpuToGpu,
            size: data.len() as u32,
            alignment: 0,
            memory_type_bits: 0,
        });

        // Copy memory
        unsafe { data.as_ptr().copy_to(transfer_buffer.get_buffer_memory().get_ptr(0), data.len()) }

        let command_buffer = create_command_buffer(&self.gfx);
        begin_command_buffer(&self.gfx, command_buffer, true);

        self.set_image_layout(&GfxImageID::new(&self.gfx, 0, 0), command_buffer, ImageLayout::TRANSFER_DST_OPTIMAL);

        let (dim_x, dim_y, dim_z) = self.image_params.image_format.dimensions();
        let region = BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource:
            ImageSubresourceLayers {
                aspect_mask: if self.image_params.pixel_format.is_depth_format() { ImageAspectFlags::DEPTH } else { ImageAspectFlags::COLOR },
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: Offset3D { x: 0, y: 0, z: 0 },
            image_extent: Extent3D { width: dim_x, height: dim_y, depth: dim_z },
        };

        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();


        let (image, _) = self.image.get(&GfxImageID::new(&self.gfx, 0, 0));

        unsafe { gfx_object!(*device).device.cmd_copy_buffer_to_image(command_buffer, transfer_buffer.as_ref().as_any().downcast_ref::<VkBuffer>().unwrap().buffer, image, ImageLayout::TRANSFER_DST_OPTIMAL, &[region]); }

        self.set_image_layout(&GfxImageID::new(&self.gfx, 0, 0), command_buffer, ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        end_command_buffer(&self.gfx, command_buffer);
        submit_command_buffer(&self.gfx, command_buffer, QueueFlags::TRANSFER);
    }

    fn get_data_size(&self) -> u32 {
        self.image_params.pixel_format.type_size() * self.image_params.image_format.pixel_count()
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

impl GfxImageBuilder<(Image, Arc<Allocation>)> for RbImage {
    fn build(&self, gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> (Image, Arc<Allocation>) {
        // Convert image details
        let (image_type, width, height, depth) = match self.create_infos.image_format {
            ImageType::Texture1d(x) => { (vk::ImageType::TYPE_1D, x, 1, 1) }
            ImageType::Texture2d(x, y) => { (vk::ImageType::TYPE_2D, x, y, 1) }
            ImageType::Texture3d(x, y, z) => { (vk::ImageType::TYPE_3D, x, y, z) }
            ImageType::Texture1dArray(x) => { (vk::ImageType::TYPE_1D, x, 1, 1) }
            ImageType::Texture2dArray(x, y) => { (vk::ImageType::TYPE_2D, x, y, 1) }
            ImageType::TextureCube(x, y) => { (vk::ImageType::TYPE_2D, x, y, 1) }
        };
        
        // Create image
        let ci_image = ImageCreateInfo {
            image_type,
            format: *VkPixelFormat::from(&self.create_infos.pixel_format),
            extent: Extent3D { width, height, depth },
            mip_levels: match self.create_infos.mip_levels {
                None => { 1 }
                Some(levels) => { levels as u32 }
            },
            array_layers: match self.create_infos.image_format {
                ImageType::TextureCube(_, _) => { 6 }
                _ => { 1 }
            },
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: VkImageUsage::from(self.create_infos.usage | ImageUsage::CopyDestination, self.create_infos.pixel_format.is_depth_format()).0,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..ImageCreateInfo::default()
        };
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        let image = vk_check!(unsafe {gfx_object!(*device).device.create_image(&ci_image, None)});

        // Allocate image memory        
        let requirements = unsafe { gfx_object!(*device).device.get_image_memory_requirements(image) };

        let allocation = gfx_object!(*device).allocator.borrow_mut().allocate(&AllocationCreateDesc {
            name: "buffer allocation",
            requirements,
            location: *VkBufferAccess::from(BufferAccess::GpuOnly),
            linear: false,
        });

        if !allocation.is_ok() {
            panic!("failed to allocate image memory");
        }

        let allocation = allocation.unwrap();

        vk_check!(unsafe { gfx_object!(*device).device.bind_image_memory(image, allocation.memory(), 0 as DeviceSize)});

        (image, Arc::new(allocation))
    }
}

pub struct RbImageView {
    images: Arc<GfxResource<(Image, Arc<Allocation>)>>,
    create_infos: ImageParams,
}

impl GfxImageBuilder<ImageView> for RbImageView {
    fn build(&self, gfx: &GfxRef, swapchain_ref: &GfxImageID) -> ImageView {
        let view_type = match self.create_infos.image_format {
            ImageType::Texture1d(_) => { ImageViewType::TYPE_1D }
            ImageType::Texture2d(_, _) => { ImageViewType::TYPE_2D }
            ImageType::Texture3d(_, _, _) => { ImageViewType::TYPE_3D }
            ImageType::Texture1dArray(_) => { ImageViewType::TYPE_1D_ARRAY }
            ImageType::Texture2dArray(_, _) => { ImageViewType::TYPE_2D_ARRAY }
            ImageType::TextureCube(_, _) => { ImageViewType::CUBE }
        };

        let (image, _) = self.images.get(&swapchain_ref);

        let ci_view = ImageViewCreateInfo {
            image,
            view_type,
            format: *VkPixelFormat::from(&self.create_infos.pixel_format),
            components: ComponentMapping { r: ComponentSwizzle::R, g: ComponentSwizzle::G, b: ComponentSwizzle::B, a: ComponentSwizzle::A },
            subresource_range: ImageSubresourceRange {
                aspect_mask: if self.create_infos.pixel_format.is_depth_format() { ImageAspectFlags::DEPTH } else { ImageAspectFlags::COLOR },
                base_mip_level: 0,
                level_count: match self.create_infos.mip_levels {
                    None => { 1 }
                    Some(levels) => { levels as u32 }
                },
                base_array_layer: 0,
                layer_count: 1,
                ..ImageSubresourceRange::default()
            },
            ..ImageViewCreateInfo::default()
        };

        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        vk_check!(unsafe { gfx_object!(*device).device.create_image_view(&ci_view, None) })
    }
}

impl VkImage {
    pub fn new(gfx: &GfxRef, create_infos: ImageCreateInfos) -> Arc<dyn GfxImage> {
        let params = create_infos.params;

        let (images, views) = if params.read_only {
            let images = Arc::new(GfxResource::new_static(gfx, Box::new(RbImage { create_infos: params })));

            (images.clone(), GfxResource::new_static(gfx, Box::new(RbImageView { create_infos: params, images })))
        } else {
            let images = Arc::new(GfxResource::new(Box::new(RbImage { create_infos: params })));
            (images.clone(), GfxResource::new(Box::new(RbImageView { create_infos: params, images })))
        };

        let image = Arc::new(Self {
            gfx: gfx.clone(),
            view: views,
            image: images,
            image_params: params,
            image_layout: RwLock::new(ImageLayout::UNDEFINED),
        });

        match create_infos.pixels {
            None => {}
            Some(pixels) => {
                image.set_data(pixels)
            }
        }

        image
    }

    pub fn from_existing_images(gfx: &GfxRef, existing_images: GfxResource<(Image, Arc<Allocation>)>, image_params: ImageParams) -> Arc<dyn GfxImage> {
        let images = Arc::new(existing_images);
        Arc::new(VkImage {
            gfx: gfx.clone(),
            image: images.clone(),
            view: GfxResource::new(Box::new(RbImageView {
                images,
                create_infos: image_params,
            })),
            image_params,
            image_layout: RwLock::new(ImageLayout::UNDEFINED),
        })
    }


    fn set_image_layout(&self, image_id: &GfxImageID, command_buffer: CommandBuffer, layout: ImageLayout) {
        let mut old_layout = self.image_layout.write().unwrap();
        let (image, _) = self.image.get(image_id);
        let mut barrier = ImageMemoryBarrier {
            old_layout: *old_layout,
            new_layout: layout,
            src_queue_family_index: QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: QUEUE_FAMILY_IGNORED,
            image,
            subresource_range:
            ImageSubresourceRange {
                aspect_mask: if self.image_params.pixel_format.is_depth_format() { ImageAspectFlags::DEPTH } else { ImageAspectFlags::COLOR },
                base_mip_level: 0,
                level_count: match self.image_params.mip_levels {
                    Some(levels) => { levels as u32 }
                    None => { 1 }
                },
                base_array_layer: 0,
                layer_count: match self.image_params.image_format {
                    ImageType::TextureCube(_, _) => { 6 }
                    _ => { 1 }
                },
            },
            ..ImageMemoryBarrier::default()
        };

        let (source_stage, destination_stage) = if *old_layout == ImageLayout::UNDEFINED && layout == ImageLayout::TRANSFER_DST_OPTIMAL
        {
            barrier.src_access_mask = AccessFlags::NONE;
            barrier.dst_access_mask = AccessFlags::TRANSFER_WRITE;

            (PipelineStageFlags::TOP_OF_PIPE, PipelineStageFlags::TRANSFER)
        } else if *old_layout == ImageLayout::TRANSFER_DST_OPTIMAL && layout == ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            barrier.src_access_mask = AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = AccessFlags::SHADER_READ;

            (PipelineStageFlags::TRANSFER, PipelineStageFlags::FRAGMENT_SHADER)
        } else {
            panic!("Unsupported layout transition");
        };

        *old_layout = layout;

        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();
        unsafe { gfx_object!(*device).device.cmd_pipeline_barrier(command_buffer, source_stage, destination_stage, DependencyFlags::empty(), &[], &[], &[barrier]); }
    }
}