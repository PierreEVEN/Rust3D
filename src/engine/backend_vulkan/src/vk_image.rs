use std::sync::Arc;

use ash::vk;
use ash::vk::{ComponentMapping, ComponentSwizzle, Extent3D, Image, ImageAspectFlags, ImageCreateInfo, ImageSubresourceRange, ImageTiling, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, SampleCountFlags, SharingMode};

use gfx::GfxRef;
use gfx::image::{GfxImage, ImageParams, ImageType, ImageUsage};
use gfx::surface::GfxImageID;
use gfx::types::{GfxCast, PixelFormat};

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check};
use crate::vk_swapchain_resource::{GfxImageBuilder, VkSwapchainResource};
use crate::vk_types::VkPixelFormat;

pub struct VkImage {
    pub image: Arc<VkSwapchainResource<Image>>,
    pub view: VkSwapchainResource<ImageView>,
    pub image_params: ImageParams,
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
}

pub struct VkImageUsage(ImageUsageFlags);

pub struct RbImage {
    create_infos: ImageParams,
}

impl GfxImageBuilder<Image> for RbImage {
    fn build(&self, gfx: &GfxRef, swapchain_ref: &GfxImageID) -> Image {
        let (image_type, width, height, depth) = match self.create_infos.image_format {
            ImageType::Texture1d(x) => { (vk::ImageType::TYPE_1D, x, 1, 1) }
            ImageType::Texture2d(x, y) => { (vk::ImageType::TYPE_2D, x, y, 1) }
            ImageType::Texture3d(x, y, z) => { (vk::ImageType::TYPE_3D, x, y, z) }
            ImageType::Texture1dArray(x) => { (vk::ImageType::TYPE_1D, x, 1, 1) }
            ImageType::Texture2dArray(x, y) => { (vk::ImageType::TYPE_2D, x, y, 1) }
            ImageType::TextureCube(x, y) => { (vk::ImageType::TYPE_2D, x, y, 1) }
        };


        let ci_image = ImageCreateInfo {
            image_type,
            format: *VkPixelFormat::from(&self.create_infos.pixel_format),
            extent: Extent3D { width, height, depth },
            mip_levels: match self.create_infos.mip_levels {
                None => { 0 }
                Some(levels) => { levels as u32 }
            },
            array_layers: match self.create_infos.image_format {
                ImageType::TextureCube(_, _) => { 6 }
                _ => { 1 }
            },
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: VkImageUsage::from(self.create_infos.usage, self.create_infos.pixel_format.is_depth_format()).0,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..ImageCreateInfo::default()
        };
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        vk_check!(unsafe {gfx_object!(*device).device.create_image(&ci_image, None)})
    }
}

pub struct RbImageView {
    images: Arc<VkSwapchainResource<Image>>,
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


        let ci_view = ImageViewCreateInfo {
            image: self.images.get(&swapchain_ref),
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

impl VkImageUsage {
    fn from(usage: ImageUsage, is_depth: bool) -> Self {
        let mut flags = ImageUsageFlags::default();
        match usage {
            ImageUsage::CopySource => { flags |= ImageUsageFlags::TRANSFER_SRC }
            ImageUsage::CopyDestination => { flags |= ImageUsageFlags::TRANSFER_DST }
            ImageUsage::Sampling => { flags |= ImageUsageFlags::SAMPLED }
            ImageUsage::GpuWriteDestination => { flags |= if is_depth { ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT } else { ImageUsageFlags::COLOR_ATTACHMENT } }
        };
        VkImageUsage(flags)
    }
}


impl VkImage {
    pub fn new(gfx: &GfxRef, create_infos: ImageParams) -> Arc<dyn GfxImage> {
        let (images, views) = if create_infos.read_only {
            let images = Arc::new(VkSwapchainResource::new_static(gfx, Box::new(RbImage { create_infos })));
            (images.clone(),
             VkSwapchainResource::new_static(gfx, Box::new(RbImageView { create_infos, images })))
        } else {
            let images = Arc::new(VkSwapchainResource::new(Box::new(RbImage { create_infos })));
            (images.clone(),
             VkSwapchainResource::new(Box::new(RbImageView { create_infos, images })))
        };

        Arc::new(Self {
            view: views,
            image: images,
            image_params: create_infos,
        })
    }
}