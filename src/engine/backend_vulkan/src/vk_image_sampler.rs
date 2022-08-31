use std::sync::Arc;
use ash::vk::{Bool32, BorderColor, CompareOp, DescriptorImageInfo, Filter, ImageLayout, ImageView, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode};
use gfx::GfxRef;

use gfx::image_sampler::{ImageSampler, SamplerCreateInfos};
use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check};

pub struct VkImageSampler {
    pub sampler: Sampler,
    pub image_infos: DescriptorImageInfo,
}

impl ImageSampler for VkImageSampler {}

impl VkImageSampler {
    pub fn new(gfx: &GfxRef, _create_infos: SamplerCreateInfos) -> Arc<Self> {
        let sampler_create_infos = SamplerCreateInfo{
            mag_filter               : Filter::LINEAR,
            min_filter               : Filter::LINEAR,
            mipmap_mode              : SamplerMipmapMode::LINEAR,
            address_mode_u            : SamplerAddressMode::REPEAT,
            address_mode_v            : SamplerAddressMode::REPEAT,
            address_mode_w            : SamplerAddressMode::REPEAT,
            mip_lod_bias              : 0.0,
            anisotropy_enable        : true as Bool32,
            max_anisotropy           : 16.0,
            compare_enable           : false as Bool32,
            compare_op               : CompareOp::ALWAYS,
            min_lod                  : 0.0,
            max_lod                  : 0.0,
            border_color             : BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates : false as Bool32,
            ..SamplerCreateInfo::default()
        };
        
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();

        let sampler = vk_check!(unsafe { gfx_object!(*device).device.create_sampler(&sampler_create_infos, None) });
        
        let image_infos = DescriptorImageInfo {
            sampler,
            image_view   : ImageView::null(),
            image_layout : ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };
        
        Arc::new(Self {
            sampler,
            image_infos
        })
    }
}