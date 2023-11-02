use std::sync::Arc;

use ash::vk;

use shader_base::{DescriptorType, ShaderStage};

use crate::{GfxVulkan, vk_check};
use crate::vk_types::VkShaderStage;

pub struct VkDescriptorType(vk::DescriptorType);

impl From<&DescriptorType> for VkDescriptorType {
    fn from(descriptor_type: &DescriptorType) -> Self {
        VkDescriptorType(match descriptor_type {
            DescriptorType::Sampler => vk::DescriptorType::SAMPLER,
            DescriptorType::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorType::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
            DescriptorType::StorageImage => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorType::UniformTexelBuffer => vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
            DescriptorType::StorageTexelBuffer => vk::DescriptorType::STORAGE_TEXEL_BUFFER,
            DescriptorType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorType::UniformBufferDynamic => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            DescriptorType::StorageBufferDynamic => vk::DescriptorType::STORAGE_BUFFER_DYNAMIC,
            DescriptorType::InputAttachment => vk::DescriptorType::INPUT_ATTACHMENT,
        })
    }
}

pub struct VkDescriptorSetLayout {
    pub descriptor_set_layout: vk::DescriptorSetLayout,
}

impl VkDescriptorSetLayout {
    pub fn new(
        name: String,
        resources: &Vec<(ShaderStage, DescriptorType, u32)>,
    ) -> Arc<Self> {
        let mut bindings = Vec::<vk::DescriptorSetLayoutBinding>::new();
        for (stage, desc_type, location) in resources {
            bindings.push(
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(*location)
                    .descriptor_type(VkDescriptorType::from(desc_type).0)
                    .descriptor_count(1)
                    .stage_flags(VkShaderStage::from(stage).flags)
                    .build(),
            );
        }

        let ci_descriptor_set_layout = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings.as_slice())
            .build();

        let descriptor_set_layout = vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .create_descriptor_set_layout(&ci_descriptor_set_layout, None)
        });
        GfxVulkan::get().set_vk_object_name(
            descriptor_set_layout,
            format!("descriptor_set_layout\t: {}", name).as_str(),
        );

        Arc::new(Self {
            descriptor_set_layout,
        })
    }
}
