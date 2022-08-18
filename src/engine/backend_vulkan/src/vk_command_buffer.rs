use ash::vk::{CommandBuffer, CommandBufferAllocateInfo, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, QueueFlags};

use gfx::GfxRef;

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check};

pub struct VkCommandPool {
    command_pool: CommandPool,
}

impl VkCommandPool {
    pub fn new(gfx: &GfxRef) -> VkCommandPool {
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        
        let create_infos = CommandPoolCreateInfo {
            flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: match gfx_object!(*device).queues.get(&QueueFlags::GRAPHICS) {
                None => {panic!("failed to find queue");}
                Some(queue) => {queue[0].index}
            },
            ..CommandPoolCreateInfo::default()
        };
        
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        let command_pool = vk_check!(unsafe {gfx_object!(*device).device.create_command_pool(&create_infos, None)});
        
        VkCommandPool {
            command_pool
        }
    }
}

pub struct VkCommandBuffer {
    pub command_buffer: CommandBuffer,
}

impl VkCommandBuffer {
    pub fn new(gfx: &GfxRef) -> VkCommandBuffer {
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        let command_pool = gfx_cast_vulkan!(gfx).command_pool.read().unwrap();

        let create_infos = CommandBufferAllocateInfo {
            command_pool: gfx_object!(*command_pool).command_pool,
            command_buffer_count: 1,
            ..CommandBufferAllocateInfo::default()
        };
        
        let command_buffer = vk_check!(unsafe { gfx_object!(*device).device.allocate_command_buffers(&create_infos) });
        
        VkCommandBuffer { command_buffer: command_buffer[0] }
    }

    pub fn start(&self) {}

    pub fn submit(&self) {}
}