use std::sync::Arc;

use ash::vk;
use gfx::GfxRef;

use gfx::shader::{DescriptorBinding, DescriptorType};

use crate::{GfxVulkan, vk_check};

pub struct VkDescriptorType(vk::DescriptorType);

impl From<&DescriptorType> for VkDescriptorType {
    fn from(descriptor_type: &DescriptorType) -> Self {
        VkDescriptorType(match descriptor_type {
            DescriptorType::Sampler => { vk::DescriptorType::SAMPLER }
            DescriptorType::CombinedImageSampler => { vk::DescriptorType::COMBINED_IMAGE_SAMPLER }
            DescriptorType::SampledImage => { vk::DescriptorType::SAMPLED_IMAGE }
            DescriptorType::StorageImage => { vk::DescriptorType::STORAGE_BUFFER }
            DescriptorType::UniformTexelBuffer => { vk::DescriptorType::UNIFORM_TEXEL_BUFFER }
            DescriptorType::StorageTexelBuffer => { vk::DescriptorType::STORAGE_TEXEL_BUFFER }
            DescriptorType::UniformBuffer => { vk::DescriptorType::UNIFORM_BUFFER }
            DescriptorType::StorageBuffer => { vk::DescriptorType::STORAGE_BUFFER }
            DescriptorType::UniformBufferDynamic => { vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC }
            DescriptorType::StorageBufferDynamic => { vk::DescriptorType::STORAGE_BUFFER_DYNAMIC }
            DescriptorType::InputAttachment => { vk::DescriptorType::INPUT_ATTACHMENT }
        })
    }
}


pub struct VkDescriptorSetLayout {
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub gfx: GfxRef,
}

impl VkDescriptorSetLayout {
    pub fn new(gfx: &GfxRef, name: String, vertex_bindings: &Vec<DescriptorBinding>, fragment_bindings: &Vec<DescriptorBinding>) -> Arc<Self> {
        let mut bindings = Vec::<vk::DescriptorSetLayoutBinding>::new();
        for binding in vertex_bindings
        {
            bindings.push(vk::DescriptorSetLayoutBinding::builder()
                .binding(binding.binding)
                .descriptor_type(VkDescriptorType::from(&binding.descriptor_type).0)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build());
        }

        for binding in fragment_bindings
        {
            bindings.push(vk::DescriptorSetLayoutBinding::builder()
                .binding(binding.binding)
                .descriptor_type(VkDescriptorType::from(&binding.descriptor_type).0)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build());
        }

        let ci_descriptor_set_layout = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings.as_slice())
            .build();

        let descriptor_set_layout = vk_check!(unsafe {gfx.cast::<GfxVulkan>().device.assume_init_ref().handle.create_descriptor_set_layout(&ci_descriptor_set_layout, None)});
        gfx.cast::<GfxVulkan>().set_vk_object_name(descriptor_set_layout, format!("descriptor_set_layout\t: {}", name).as_str());
        
        Arc::new(Self {
            descriptor_set_layout,
            gfx: gfx.clone(),
        })
    }
}