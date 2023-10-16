use std::collections::HashMap;
use crate::{BindPoint, DescriptorType};

use rspirv_reflect::{Reflection};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct DescriptorBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
}

pub struct SpirvReflector {
    pub bindings: HashMap<BindPoint, DescriptorBinding>,
    pub push_constant_size: Option<u32>,
    pub compute_group_size: Option<(u32, u32, u32)>
}

impl SpirvReflector {
    pub fn new(spirv_code: &Vec<u32>) -> Result<SpirvReflector, String> {
        let info = match Reflection::new_from_spirv(unsafe { std::slice::from_raw_parts(spirv_code.as_ptr() as *const u8, spirv_code.len() * 4) }) {
            Ok(reflection) => { reflection }
            Err(_) => { panic!("failed to get reflection data") }
        };
        let mut bindings = HashMap::new();
        match info.get_descriptor_sets() {
            Ok(desc_sets) => {
                for (_, data) in desc_sets {
                    for (sub_id, sub_data) in data {
                        if bindings.contains_key(&BindPoint::new(sub_data.name.as_str())) {
                            return Err(format!("Duplicated binding : {}", sub_data.name));
                        }
                        bindings.insert(BindPoint::new(sub_data.name.as_str()), 
                                        DescriptorBinding {
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
                                _ => { panic!("unhandled binding type"); }
                            },
                        });
                    }
                }
            }
            Err(_) => { panic!("failed to get reflection data") }
        }
        
        let mut push_constant_size = None;
        match info.get_push_constant_range() {
            Ok(push_constants) => {
                if let Some(push_constant) = push_constants {
                    push_constant_size = Some(push_constant.size);
                }
            }
            Err(_) => { panic!("failed to get reflection data") }
        }

        Ok(Self { bindings, push_constant_size, compute_group_size: info.get_compute_group_size() })
    }
}