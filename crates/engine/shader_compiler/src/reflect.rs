use std::slice;

use rspirv_reflect::{Reflection};

use gfx::shader::{DescriptorBinding, DescriptorType};
use gfx::shader_instance::BindPoint;

pub struct SpirvReflector {
    pub bindings: Vec<DescriptorBinding>,
    pub push_constant_size: u32,
}

impl SpirvReflector {
    pub fn new(spirv_code: &Vec<u32>) -> SpirvReflector {
        let info = match Reflection::new_from_spirv(unsafe { slice::from_raw_parts(spirv_code.as_ptr() as *const u8, spirv_code.len() * 4) }) {
            Ok(reflection) => { reflection }
            Err(_) => { logger::fatal!("failed to get reflection data") }
        };

        let mut bindings = Vec::new();
        match info.get_descriptor_sets() {
            Ok(desc_sets) => {
                for (_, data) in desc_sets {
                    for (sub_id, sub_data) in data {
                        bindings.push(DescriptorBinding {
                            bind_point: BindPoint::new(sub_data.name.as_str()),
                            binding: sub_id,
                            descriptor_type: match sub_data.ty {
                                rspirv_reflect::DescriptorType::SAMPLER => { DescriptorType::Sampler }
                                rspirv_reflect::DescriptorType::COMBINED_IMAGE_SAMPLER => { DescriptorType::CombinedImageSampler }
                                rspirv_reflect::DescriptorType::SAMPLED_IMAGE => { DescriptorType::SampledImage }
                                rspirv_reflect::DescriptorType::STORAGE_IMAGE => { DescriptorType::StorageImage }
                                rspirv_reflect::DescriptorType::UNIFORM_TEXEL_BUFFER => { DescriptorType::UniformTexelBuffer }
                                rspirv_reflect::DescriptorType::STORAGE_TEXEL_BUFFER => { DescriptorType::StorageTexelBuffer }
                                rspirv_reflect::DescriptorType::UNIFORM_BUFFER => { DescriptorType::UniformBuffer }
                                rspirv_reflect::DescriptorType::STORAGE_BUFFER => { DescriptorType::StorageBuffer }
                                rspirv_reflect::DescriptorType::UNIFORM_BUFFER_DYNAMIC => { DescriptorType::UniformBufferDynamic }
                                rspirv_reflect::DescriptorType::STORAGE_BUFFER_DYNAMIC => { DescriptorType::StorageBufferDynamic }
                                rspirv_reflect::DescriptorType::INPUT_ATTACHMENT => { DescriptorType::InputAttachment }
                                _ => { logger::fatal!("unhandled binding type"); }
                            },
                        });
                    }
                }
            }
            Err(_) => { logger::fatal!("failed to get reflection data") }
        }
        
        let mut push_constant_size: u32 = 0;
        match info.get_push_constant_range() {
            Ok(push_constants) => {
                if let Some(push_constant) = push_constants {
                    push_constant_size = push_constant.size;
                }
            }
            Err(_) => { logger::fatal!("failed to get reflection data") }
        }

        SpirvReflector { bindings, push_constant_size }
    }
}