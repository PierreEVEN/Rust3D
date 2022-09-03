use std::sync::{Arc, RwLock};

use ash::vk::{CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsageFlags, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, PipelineBindPoint, QueueFlags, SubmitInfo};

use gfx::buffer::GfxBuffer;
use gfx::command_buffer::GfxCommandBuffer;
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::shader::{PassID, ShaderProgram};
use gfx::shader_instance::ShaderInstance;
use gfx::surface::GfxImageID;

use crate::{gfx_cast_vulkan, GfxVulkan, vk_check, VkShaderInstance, VkShaderProgram};

pub struct VkCommandPool {
    pub command_pool: CommandPool,
}

pub fn create_command_buffer(gfx: &GfxRef) -> CommandBuffer {
    let device = &gfx_cast_vulkan!(gfx).device;
    let command_pool = &gfx_cast_vulkan!(gfx).command_pool;
    let create_infos = CommandBufferAllocateInfo {
        command_pool: (*command_pool).command_pool,
        command_buffer_count: 1,
        level: CommandBufferLevel::PRIMARY,
        ..CommandBufferAllocateInfo::default()
    };
    vk_check!(unsafe { (*device).device.allocate_command_buffers(&create_infos) })[0]
}

pub fn begin_command_buffer(gfx: &GfxRef, command_buffer: CommandBuffer, one_time: bool) {
    let device = &gfx_cast_vulkan!(gfx).device;
    vk_check!(unsafe { (*device).device.begin_command_buffer(command_buffer, &CommandBufferBeginInfo { 
        flags: if one_time { CommandBufferUsageFlags::ONE_TIME_SUBMIT } else { CommandBufferUsageFlags::empty() },
        ..CommandBufferBeginInfo::default() 
    })})
}

pub fn end_command_buffer(gfx: &GfxRef, command_buffer: CommandBuffer) {
    let device = &gfx_cast_vulkan!(gfx).device;
    vk_check!(unsafe { (*device).device.end_command_buffer(command_buffer)})
}

pub fn submit_command_buffer(gfx: &GfxRef, command_buffer: CommandBuffer, queue_flags: QueueFlags) {
    let device = &gfx_cast_vulkan!(gfx).device;
    match (*device).get_queue(queue_flags) {
        Ok(queue) => {
            queue.submit(SubmitInfo {
                command_buffer_count: 1,
                p_command_buffers: &command_buffer,
                ..SubmitInfo::default()
            });
        }
        Err(_) => {
            panic!("failed to find queue");
        }
    }
}


impl VkCommandPool {
    pub fn new(gfx: &GfxRef) -> VkCommandPool {
        let device = &gfx_cast_vulkan!(gfx).device;

        let create_infos = CommandPoolCreateInfo {
            flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: match (*device).queues.get(&QueueFlags::GRAPHICS) {
                None => { panic!("failed to find queue"); }
                Some(queue) => { queue[0].index }
            },
            ..CommandPoolCreateInfo::default()
        };

        let device = &gfx_cast_vulkan!(gfx).device;
        let command_pool = vk_check!(unsafe {(*device).device.create_command_pool(&create_infos, None)});

        VkCommandPool {
            command_pool
        }
    }
}

pub struct VkCommandBuffer {
    pub command_buffer: GfxResource<CommandBuffer>,
    gfx: GfxRef,
    pass_id: RwLock<PassID>,
    image_id: RwLock<GfxImageID>,
}

pub struct RbCommandBuffer {}

impl GfxImageBuilder<CommandBuffer> for RbCommandBuffer {
    fn build(&self, gfx: &GfxRef, _: &GfxImageID) -> CommandBuffer {
        create_command_buffer(gfx)
    }
}

impl VkCommandBuffer {
    pub fn new(gfx: &GfxRef) -> Arc<VkCommandBuffer> {
        Arc::new(VkCommandBuffer {
            command_buffer: GfxResource::new(Box::new(RbCommandBuffer {})),
            gfx: gfx.clone(),
            pass_id: RwLock::new(PassID::new("undefined")),
            image_id: RwLock::new(GfxImageID::new(gfx, 0, 0)),
        })
    }

    pub fn init_for(&self, new_id: PassID, image_id: GfxImageID) {
        *self.pass_id.write().unwrap() = new_id;
        *self.image_id.write().unwrap() = image_id;
    }
}

impl GfxCommandBuffer for VkCommandBuffer {
    fn bind_program(&self, program: &Arc<dyn ShaderProgram>) {
        let device = &gfx_cast_vulkan!(self.gfx).device;
        let vk_program = program.as_ref().as_any().downcast_ref::<VkShaderProgram>().unwrap();
        unsafe { (*device).device.cmd_bind_pipeline(self.command_buffer.get(&*self.image_id.read().unwrap()), PipelineBindPoint::GRAPHICS, vk_program.pipeline); }
    }

    fn bind_shader_instance(&self, instance: &Arc<dyn ShaderInstance>) {
        let device = &gfx_cast_vulkan!(self.gfx).device;
        let vk_shader_instance = instance.as_ref().as_any().downcast_ref::<VkShaderInstance>().unwrap();
        vk_shader_instance.refresh_descriptors(&*self.image_id.read().unwrap());
        unsafe {
            (*device).device.cmd_bind_descriptor_sets(
                self.command_buffer.get(&*self.image_id.read().unwrap()),
                PipelineBindPoint::GRAPHICS,
                *vk_shader_instance.pipeline_layout,
                0,
                &[vk_shader_instance.descriptor_sets.read().unwrap().get(&*self.image_id.read().unwrap())],
                &[],
            );
        }
    }

    fn draw_mesh(&self, _mesh: &Arc<dyn GfxBuffer>, _instance_count: u32, _first_instance: u32) {
        todo!()
    }

    fn draw_mesh_advanced(&self, _mesh: &Arc<dyn GfxBuffer>, _first_index: u32, _vertex_offset: u32, _index_count: u32, _instance_count: u32, _first_instance: u32) {
        todo!()
    }

    fn draw_mesh_indirect(&self, _mesh: &Arc<dyn GfxBuffer>) {
        todo!()
    }

    fn draw_procedural(&self, vertex_count: u32, first_vertex: u32, instance_count: u32, first_instance: u32) {
        let device = &gfx_cast_vulkan!(self.gfx).device;
        unsafe { (*device).device.cmd_draw(self.command_buffer.get(&*self.image_id.read().unwrap()), vertex_count, instance_count, first_vertex, first_instance) }
    }

    fn set_scissor(&self) {
        todo!()
    }

    fn push_constant(&self) {
        todo!()
    }

    fn get_pass_id(&self) -> PassID {
        self.pass_id.read().unwrap().clone()
    }
}