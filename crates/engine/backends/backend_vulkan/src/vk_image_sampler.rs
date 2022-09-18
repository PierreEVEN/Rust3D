use std::sync::Arc;
use ash::vk;
use gfx::GfxRef;

use gfx::image_sampler::{ImageSampler, SamplerCreateInfos};
use crate::{GfxVulkan, vk_check};

pub struct VkImageSampler {
    pub sampler: vk::Sampler,
    pub sampler_info: vk::DescriptorImageInfo,
}

impl ImageSampler for VkImageSampler {}

impl VkImageSampler {
    pub fn new(gfx: &GfxRef, _create_infos: SamplerCreateInfos) -> Arc<Self> {
        let sampler_create_infos = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .mip_lod_bias(0.0)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .min_lod(0.0)
            .max_lod(0.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .build();
        
        let sampler = vk_check!(unsafe { gfx.cast::<GfxVulkan>().device.handle.create_sampler(&sampler_create_infos, None) });
        
        let sampler_info = vk::DescriptorImageInfo::builder()
            .sampler(sampler)
            .image_view(vk::ImageView::null())
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build();
        
        Arc::new(Self {
            sampler,
            sampler_info
        })
    }
}