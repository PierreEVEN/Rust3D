use std::sync::Arc;
use ash::vk::{CommandBuffer, CommandBufferAllocateInfo, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, QueueFlags, RenderPassBeginInfo, SubpassContents};
use gfx::buffer::GfxBuffer;

use gfx::command_buffer::GfxCommandBuffer;
use gfx::GfxRef;
use gfx::render_pass::RenderPassInstance;
use gfx::shader::ShaderProgram;

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check};

pub struct VkCommandPool {
    pub command_pool: CommandPool,
}

impl VkCommandPool {
    pub fn new(gfx: &GfxRef) -> VkCommandPool {
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();

        let create_infos = CommandPoolCreateInfo {
            flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: match gfx_object!(*device).queues.get(&QueueFlags::GRAPHICS) {
                None => { panic!("failed to find queue"); }
                Some(queue) => { queue[0].index }
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
    gfx: GfxRef,
    
}

impl VkCommandBuffer {
    pub fn new(gfx: &GfxRef) -> Arc<VkCommandBuffer> {
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        let command_pool = gfx_cast_vulkan!(gfx).command_pool.read().unwrap();

        let create_infos = CommandBufferAllocateInfo {
            command_pool: gfx_object!(*command_pool).command_pool,
            command_buffer_count: 1,
            ..CommandBufferAllocateInfo::default()
        };

        let command_buffer = vk_check!(unsafe { gfx_object!(*device).device.allocate_command_buffers(&create_infos) });

        Arc::new(VkCommandBuffer { command_buffer: command_buffer[0], gfx: gfx.clone() })
    }
}

impl GfxCommandBuffer for VkCommandBuffer {
    fn bind_program(&self, material: Arc<dyn ShaderProgram>) {
        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();

        // gfx_object!(*device).device.cmd_bind_descriptor_sets()
    }

    fn draw_mesh(&self, mesh: Arc<dyn GfxBuffer>, instance_count: u32, first_instance: u32) {
        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();
        //gfx_object!(*device).device.draw
        todo!()
    }

    fn draw_mesh_advanced(&self, mesh: Arc<dyn GfxBuffer>, first_index: u32, vertex_offset: u32, index_count: u32, instance_count: u32, first_instance: u32) {
        todo!()
    }

    fn draw_mesh_indirect(&self, mesh: Arc<dyn GfxBuffer>) {
        todo!()
    }

    fn draw_procedural(&self, vertex_count: u32, first_vertex: u32, instance_count: u32, first_instance: u32) {
        todo!()
    }

    fn set_scissor(&self) {
        todo!()
    }

    fn push_constant(&self) {
        todo!()
    }

    fn init(&self) {
        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();

        vk_check!(unsafe { gfx_object!(*device).device.begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default()) });
    }

    fn submit(&self) {

        let submit_infos = SubmitInfo {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: &PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: 1,
            p_signal_semaphores: &self.render_finished_semaphore.get(&self.surface.get_current_ref()),
            ..SubmitInfo::default()
        };

        gfx_object!(*device).get_queue(QueueFlags::GRAPHICS).unwrap().submit(submit_infos);
    }

    fn begin_pass(&self, pass: &dyn RenderPassInstance) {
        let begin_infos = RenderPassBeginInfo {
            render_pass: self.owner.as_ref().as_any().downcast_ref::<VkRenderPass>().expect("invalid render pass").render_pass,
            framebuffer: self._framebuffers.get(&self.surface.get_current_ref()),
            render_area: vk::Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: Extent2D { width: res.x, height: res.y },
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..RenderPassBeginInfo::default()
        };
        unsafe { gfx_object!(*device).device.cmd_begin_render_pass(command_buffer, &begin_infos, SubpassContents::INLINE) };
    }

    fn end_pass(&self, pass: &dyn RenderPassInstance) {
        todo!()
    }
}