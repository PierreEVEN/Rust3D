use std::collections::HashMap;
use std::sync::RwLock;
use std::thread;
use std::thread::ThreadId;

use ash::vk;

use crate::{vk_check, GfxVulkan};

pub struct VkDescriptorPool {
    pools: RwLock<HashMap<ThreadId, Vec<(vk::DescriptorPool, u32)>>>,
    max_descriptor_per_pool: u32,
    max_descriptor_per_type: u32,
}

impl VkDescriptorPool {
    pub fn new(max_descriptor_per_pool: u32, max_descriptor_per_type: u32) -> Self {
        Self {
            pools: RwLock::default(),
            max_descriptor_per_pool,
            max_descriptor_per_type,
        }
    }

    fn fetch_available_pool(&self) -> vk::DescriptorPool {
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
        pools
            .get_mut(&thread::current().id())
            .unwrap()
            .push((pool, 1));
        pool
    }

    fn create_descriptor_pool(&self) -> vk::DescriptorPool {
        let pool_sizes = vec![
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLER,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLED_IMAGE,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_TEXEL_BUFFER,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER_DYNAMIC,
                descriptor_count: self.max_descriptor_per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::INPUT_ATTACHMENT,
                descriptor_count: self.max_descriptor_per_type,
            },
        ];

        GfxVulkan::get().set_vk_object_name(
            vk_check!(unsafe {
                GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .handle
                    .create_descriptor_pool(
                        &vk::DescriptorPoolCreateInfo::builder()
                            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
                            .max_sets(self.max_descriptor_per_pool)
                            .pool_sizes(pool_sizes.as_slice())
                            .build(),
                        None,
                    )
            }),
            format!(
                "descriptor pool\t\t: thread@{}",
                thread::current().name().unwrap()
            )
            .as_str(),
        )
    }

    pub fn allocate(&self, name: String, layout: vk::DescriptorSetLayout) -> vk::DescriptorSet {
        let descriptor_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.fetch_available_pool())
            .set_layouts(&[layout])
            .build();
        let device = &GfxVulkan::get().device;

        GfxVulkan::get().set_vk_object_name(
            vk_check!(unsafe {
                device
                    .assume_init_ref()
                    .handle
                    .allocate_descriptor_sets(&descriptor_info)
            })[0],
            format!("descriptor set\t\t: {}", name).as_str(),
        )
    }
}
