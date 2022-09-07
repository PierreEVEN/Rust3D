use std::sync::{Arc, RwLock};

use ash::vk::{CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsageFlags, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Extent2D, Offset2D, PipelineBindPoint, QueueFlags, Rect2D, ShaderStageFlags, SubmitInfo};

use gfx::buffer::{BufferMemory, GfxBuffer};
use gfx::command_buffer::GfxCommandBuffer;
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::shader::{PassID, ShaderProgram, ShaderStage};
use gfx::shader_instance::ShaderInstance;
use gfx::surface::{GfxImageID, GfxSurface};
use gfx::types::Scissors;

use crate::{GfxVulkan, vk_check, VkShaderInstance, VkShaderProgram};

pub struct VkCommandPool {
    pub command_pool: CommandPool,
}

pub fn create_command_buffer(gfx: &GfxRef) -> CommandBuffer {
    let create_infos = CommandBufferAllocateInfo {
        command_pool: gfx.cast::<GfxVulkan>().command_pool.command_pool,
        command_buffer_count: 1,
        level: CommandBufferLevel::PRIMARY,
        ..CommandBufferAllocateInfo::default()
    };
    vk_check!(unsafe { gfx.cast::<GfxVulkan>().device.handle.allocate_command_buffers(&create_infos) })[0]
}

pub fn begin_command_buffer(gfx: &GfxRef, command_buffer: CommandBuffer, one_time: bool) {
    vk_check!(unsafe { gfx.cast::<GfxVulkan>().device.handle.begin_command_buffer(command_buffer, &CommandBufferBeginInfo { 
        flags: if one_time { CommandBufferUsageFlags::ONE_TIME_SUBMIT } else { CommandBufferUsageFlags::empty() },
        ..CommandBufferBeginInfo::default() 
    })})
}

pub fn end_command_buffer(gfx: &GfxRef, command_buffer: CommandBuffer) {
    vk_check!(unsafe { gfx.cast::<GfxVulkan>().device.handle.end_command_buffer(command_buffer)})
}

pub fn submit_command_buffer(gfx: &GfxRef, command_buffer: CommandBuffer, queue_flags: QueueFlags) {
    match gfx.cast::<GfxVulkan>().device.get_queue(queue_flags) {
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
        let create_infos = CommandPoolCreateInfo {
            flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: match gfx.cast::<GfxVulkan>().device.queues.get(&QueueFlags::GRAPHICS) {
                None => { panic!("failed to find queue"); }
                Some(queue) => { queue[0].index }
            },
            ..CommandPoolCreateInfo::default()
        };

        let command_pool = vk_check!(unsafe {gfx.cast::<GfxVulkan>().device.handle.create_command_pool(&create_infos, None)});

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
    surface: Arc<dyn GfxSurface>,
}

pub struct RbCommandBuffer {}

impl GfxImageBuilder<CommandBuffer> for RbCommandBuffer {
    fn build(&self, gfx: &GfxRef, _: &GfxImageID) -> CommandBuffer {
        create_command_buffer(gfx)
    }
}

impl VkCommandBuffer {
    pub fn new(gfx: &GfxRef, surface: &Arc<dyn GfxSurface>) -> Arc<VkCommandBuffer> {
        Arc::new(VkCommandBuffer {
            command_buffer: GfxResource::new(gfx, RbCommandBuffer {}),
            gfx: gfx.clone(),
            pass_id: RwLock::new(PassID::new("undefined")),
            image_id: RwLock::new(GfxImageID::null()),
            surface: surface.clone(),
        })
    }

    pub fn init_for(&self, new_id: PassID, image_id: GfxImageID) {
        *self.pass_id.write().unwrap() = new_id;
        *self.image_id.write().unwrap() = image_id;
    }
}

impl GfxCommandBuffer for VkCommandBuffer {
    fn bind_program(&self, program: &Arc<dyn ShaderProgram>) {
        unsafe {
            self.gfx.cast::<GfxVulkan>().device.handle.cmd_bind_pipeline(
                self.command_buffer.get(&*self.image_id.read().unwrap()),
                PipelineBindPoint::GRAPHICS,
                program.cast::<VkShaderProgram>().pipeline,
            );
        }
    }

    fn bind_shader_instance(&self, instance: &Arc<dyn ShaderInstance>) {
        instance.cast::<VkShaderInstance>().refresh_descriptors(&*self.image_id.read().unwrap());
        unsafe {
            self.gfx.cast::<GfxVulkan>().device.handle.cmd_bind_descriptor_sets(
                self.command_buffer.get(&*self.image_id.read().unwrap()),
                PipelineBindPoint::GRAPHICS,
                *instance.cast::<VkShaderInstance>().pipeline_layout,
                0,
                &[instance.cast::<VkShaderInstance>().descriptor_sets.read().unwrap().get(&*self.image_id.read().unwrap())],
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
        unsafe { self.gfx.cast::<GfxVulkan>().device.handle.cmd_draw(self.command_buffer.get(&*self.image_id.read().unwrap()), vertex_count, instance_count, first_vertex, first_instance) }
    }

    fn set_scissor(&self, scissors: Scissors) {
        unsafe {
            self.gfx.cast::<GfxVulkan>().device.handle.cmd_set_scissor(self.command_buffer.get(&*self.image_id.read().unwrap()), 0, &[Rect2D {
                extent: Extent2D { width: scissors.width, height: scissors.height },
                offset: Offset2D { x: scissors.min_x, y: scissors.min_y },
            }])
        }
        todo!()
    }

    fn push_constant(&self, program: &Arc<dyn ShaderProgram>, data: BufferMemory, stage: ShaderStage) {
        unsafe {
            self.gfx.cast::<GfxVulkan>().device.handle.cmd_push_constants(self.command_buffer.get(&*self.image_id.read().unwrap()), *program.cast::<VkShaderProgram>().pipeline_layout, match stage {
                ShaderStage::Vertex => { ShaderStageFlags::VERTEX }
                ShaderStage::Fragment => { ShaderStageFlags::FRAGMENT }
            }, 0, data.as_slice())
        }
    }


    fn get_pass_id(&self) -> PassID {
        self.pass_id.read().unwrap().clone()
    }

    fn get_surface(&self) -> Arc<dyn GfxSurface> {
        self.surface.clone()
    }
}