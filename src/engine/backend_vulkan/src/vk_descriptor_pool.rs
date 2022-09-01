use std::collections::HashMap;
use std::sync::RwLock;
use std::thread;
use std::thread::{ThreadId};

use ash::vk::{DescriptorPool, DescriptorPoolCreateFlags, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorType};

use gfx::GfxRef;

use crate::{gfx_cast_vulkan, GfxVulkan, vk_check};

pub struct VkDescriptorPool {
    gfx: GfxRef,
    pools: RwLock<HashMap<ThreadId, Vec<(DescriptorPool, u32)>>>,
    max_descriptor_per_pool: u32,
    max_descriptor_per_type: u32,
}

impl VkDescriptorPool {
    pub fn new(gfx: &GfxRef, max_descriptor_per_pool: u32, max_descriptor_per_type: u32) -> Self { Self { gfx: gfx.clone(), pools: RwLock::default(), max_descriptor_per_pool, max_descriptor_per_type } }

    fn fetch_available_pool(&self) -> DescriptorPool {
        let mut pools = self.pools.write().unwrap();
        match pools.get_mut(&thread::current().id()) {
            None => {
                let pool = self.create_descriptor_pool();
                pools.insert(thread::current().id(), Vec::from([(pool, 0)]));
                return pool;
            }
            Some(pools) => {
                for (pool, usage) in pools {
                    if *usage < self.max_descriptor_per_pool {
                        *usage += 1;
                        return *pool;
                    }
                }
            }
        }
        let pool = self.create_descriptor_pool();
        pools.get_mut(&thread::current().id()).unwrap().push((pool, 1));
        pool
    }

    fn create_descriptor_pool(&self) -> DescriptorPool {
        let pool_sizes = vec![
            DescriptorPoolSize { ty: DescriptorType::SAMPLER, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::COMBINED_IMAGE_SAMPLER, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::SAMPLED_IMAGE, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::STORAGE_IMAGE, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::UNIFORM_TEXEL_BUFFER, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::STORAGE_TEXEL_BUFFER, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::UNIFORM_BUFFER, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::STORAGE_BUFFER, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::UNIFORM_BUFFER_DYNAMIC, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::STORAGE_BUFFER_DYNAMIC, descriptor_count: self.max_descriptor_per_type },
            DescriptorPoolSize { ty: DescriptorType::INPUT_ATTACHMENT, descriptor_count: self.max_descriptor_per_type },
        ];

        vk_check!(unsafe {
                    gfx_cast_vulkan!(self.gfx).device.device.create_descriptor_pool(&DescriptorPoolCreateInfo {
                        flags: DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET | DescriptorPoolCreateFlags::UPDATE_AFTER_BIND,
                        max_sets: self.max_descriptor_per_pool,
                        pool_size_count: pool_sizes.len() as u32,
                        p_pool_sizes: pool_sizes.as_ptr(),
                        ..DescriptorPoolCreateInfo::default()
                    }, None)
                })
    }

    pub fn allocate(&self, layout: &DescriptorSetLayout) -> DescriptorSet {
        let descriptor_info = DescriptorSetAllocateInfo {
            descriptor_pool: self.fetch_available_pool(),
            descriptor_set_count: 1,
            p_set_layouts: layout as *const DescriptorSetLayout,
            ..DescriptorSetAllocateInfo::default()
        };
        let device = &gfx_cast_vulkan!(self.gfx).device;

        vk_check!(unsafe { (*device).device.allocate_descriptor_sets(&descriptor_info) })[0]
    }
}