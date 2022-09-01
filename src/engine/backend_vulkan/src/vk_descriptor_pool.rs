use ash::vk::{DescriptorPool, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout};

use gfx::GfxRef;

use crate::{gfx_cast_vulkan, GfxVulkan, vk_check};

pub struct VkDescriptorPool {
    gfx: GfxRef,
}

impl VkDescriptorPool {
    pub fn new(gfx: &GfxRef) -> Self { Self { gfx: gfx.clone() } }

    pub fn allocate(&self, layout: &DescriptorSetLayout) -> DescriptorSet {
        let descriptor_info = DescriptorSetAllocateInfo {
            descriptor_pool: DescriptorPool::null(),
            descriptor_set_count: 1,
            p_set_layouts: layout as *const DescriptorSetLayout,
            ..DescriptorSetAllocateInfo::default()
        };
        let device = &gfx_cast_vulkan!(self.gfx).device;

        vk_check!(unsafe { (*device).device.allocate_descriptor_sets(&descriptor_info) })[0]
    }
}