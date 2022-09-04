﻿use std::ptr::null;
use std::sync::Arc;

use ash::vk::{DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, ShaderStageFlags};
use gfx::GfxRef;

use gfx::shader::{DescriptorBinding, DescriptorType};

use crate::{GfxVulkan, vk_check};

pub struct VkDescriptorType(ash::vk::DescriptorType);

impl From<&DescriptorType> for VkDescriptorType {
    fn from(descriptor_type: &DescriptorType) -> Self {
        VkDescriptorType(match descriptor_type {
            DescriptorType::Sampler => { ash::vk::DescriptorType::SAMPLER }
            DescriptorType::CombinedImageSampler => { ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER }
            DescriptorType::SampledImage => { ash::vk::DescriptorType::SAMPLED_IMAGE }
            DescriptorType::StorageImage => { ash::vk::DescriptorType::STORAGE_BUFFER }
            DescriptorType::UniformTexelBuffer => { ash::vk::DescriptorType::UNIFORM_TEXEL_BUFFER }
            DescriptorType::StorageTexelBuffer => { ash::vk::DescriptorType::STORAGE_TEXEL_BUFFER }
            DescriptorType::UniformBuffer => { ash::vk::DescriptorType::UNIFORM_BUFFER }
            DescriptorType::StorageBuffer => { ash::vk::DescriptorType::STORAGE_BUFFER }
            DescriptorType::UniformBufferDynamic => { ash::vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC }
            DescriptorType::StorageBufferDynamic => { ash::vk::DescriptorType::STORAGE_BUFFER_DYNAMIC }
            DescriptorType::InputAttachment => { ash::vk::DescriptorType::INPUT_ATTACHMENT }
        })
    }
}


pub struct VkDescriptorSetLayout {
    pub descriptor_set_layout: DescriptorSetLayout,
    pub gfx: GfxRef,
}

impl VkDescriptorSetLayout {
    pub fn new(gfx: &GfxRef, vertex_bindings: &Vec<DescriptorBinding>, fragment_bindings: &Vec<DescriptorBinding>) -> Arc<Self> {
        let mut bindings = Vec::<DescriptorSetLayoutBinding>::new();
        for binding in vertex_bindings
        {
            bindings.push(DescriptorSetLayoutBinding {
                binding: binding.binding,
                descriptor_type: VkDescriptorType::from(&binding.descriptor_type).0,
                descriptor_count: 1,
                stage_flags: ShaderStageFlags::VERTEX,
                p_immutable_samplers: null(),
                ..DescriptorSetLayoutBinding::default()
            });
        }

        for binding in fragment_bindings
        {
            bindings.push(DescriptorSetLayoutBinding {
                binding: binding.binding,
                descriptor_type: VkDescriptorType::from(&binding.descriptor_type).0,
                descriptor_count: 1,
                stage_flags: ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: null(),
                ..DescriptorSetLayoutBinding::default()
            });
        }

        let ci_descriptor_set_layout = DescriptorSetLayoutCreateInfo {
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
            ..DescriptorSetLayoutCreateInfo::default()
        };

        let device = &gfx.cast::<GfxVulkan>().device;
        let descriptor_set_layout = vk_check!(unsafe {device.handle.create_descriptor_set_layout(&ci_descriptor_set_layout, None)});

        Arc::new(Self {
            descriptor_set_layout,
            gfx: gfx.clone(),
        })
    }
}

pub struct VkDescriptorSet {}

impl VkDescriptorSet {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}