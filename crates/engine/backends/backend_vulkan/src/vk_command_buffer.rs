use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::ThreadId;

use ash::vk;
use ash::vk::CommandPool;

use shader_base::ShaderStage;
use shader_base::types::Scissors;
use core::gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use core::gfx::surface::{Frame};
use core::gfx::mesh::{Mesh, IndexBufferType};
use core::gfx::command_buffer::{GfxCommandBuffer, CommandCtx};
use core::gfx::shader::{ShaderProgram};
use core::gfx::shader_instance::{ShaderInstance};
use core::gfx::buffer::BufferMemory;

use crate::{vk_check, GfxVulkan, VkBuffer, VkShaderInstance, VkShaderProgram};

#[derive(Default)]
pub struct VkCommandPool {
    command_pool: RwLock<HashMap<ThreadId, CommandPool>>,
}

pub fn create_command_buffer(name: String) -> vk::CommandBuffer {
    let create_infos = vk::CommandBufferAllocateInfo::builder()
        .command_pool(unsafe { GfxVulkan::get().command_pool.assume_init_ref() }.get_for_current_thread())
        .command_buffer_count(1)
        .level(vk::CommandBufferLevel::PRIMARY)
        .build();
    GfxVulkan::get().set_vk_object_name(
        vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .allocate_command_buffers(&create_infos)
        })[0],
        format!("command_buffer\t: {}", name).as_str(),
    )
}

pub fn begin_command_buffer(command_buffer: vk::CommandBuffer, one_time: bool) {
    vk_check!(unsafe {
        GfxVulkan::get()
            .device
            .assume_init_ref()
            .handle
            .begin_command_buffer(
                command_buffer,
                &vk::CommandBufferBeginInfo::builder()
                    .flags(if one_time {
                        vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT
                    } else {
                        vk::CommandBufferUsageFlags::empty()
                    })
                    .build(),
            )
    });
}

pub fn end_command_buffer(command_buffer: vk::CommandBuffer) {
    vk_check!(unsafe {
        GfxVulkan::get()
            .device
            .assume_init_ref()
            .handle
            .end_command_buffer(command_buffer)
    })
}

pub fn submit_command_buffer(command_buffer: vk::CommandBuffer, queue_flags: vk::QueueFlags) {
    unsafe {
        match GfxVulkan::get()
            .device
            .assume_init_ref()
            .get_queue(queue_flags)
        {
            Ok(queue) => {
                queue.submit(
                    vk::SubmitInfo::builder()
                        .command_buffers(&[command_buffer])
                        .build(),
                );
            }
            Err(_) => {
                logger::fatal!("failed to find queue");
            }
        }
    }
}

impl VkCommandPool {
    pub fn get_for_current_thread(&self) -> CommandPool {
        match self.command_pool.read().unwrap().get(&thread::current().id()) {
            None => {}
            Some(pool) => {
                return pool.clone();
            }
        }

        let create_infos = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(
                match unsafe { GfxVulkan::get().device.assume_init_ref() }
                    .queues
                    .get(&vk::QueueFlags::GRAPHICS)
                {
                    None => {
                        logger::fatal!("failed to find queue");
                    }
                    Some(queue) => queue[0].index,
                },
            )
            .build();

        let command_pool = vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .create_command_pool(&create_infos, None)
        });

        GfxVulkan::get()
            .set_vk_object_name(command_pool, format!("command pool\t\t: {:?}", thread::current().id()).as_str());

        self.command_pool.write().unwrap().insert(thread::current().id(), command_pool.clone());

        command_pool
    }
}

pub struct VkCommandBuffer {
    pub command_buffer: GfxResource<vk::CommandBuffer>,
}

pub struct RbCommandBuffer {
    name: String,
}

impl GfxImageBuilder<vk::CommandBuffer> for RbCommandBuffer {
    fn build(&self, _: &Frame) -> vk::CommandBuffer {
        create_command_buffer(self.name.clone())
    }
}

impl VkCommandBuffer {
    pub fn new(name: String) -> Arc<VkCommandBuffer> {
        Arc::new(VkCommandBuffer {
            command_buffer: GfxResource::new(RbCommandBuffer { name })
        })
    }
}

impl GfxCommandBuffer for VkCommandBuffer {
    fn bind_program(&self, ctx: &CommandCtx, program: &Arc<dyn ShaderProgram>) {
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_bind_pipeline(
                    self.command_buffer.get(ctx.frame()),
                    vk::PipelineBindPoint::GRAPHICS,
                    program.cast::<VkShaderProgram>().pipeline,
                );
        }
    }

    fn bind_shader_instance(&self, ctx: &CommandCtx, instance: &Arc<dyn ShaderInstance>) {
        instance
            .cast::<VkShaderInstance>()
            .refresh_descriptors(ctx.frame(), ctx.render_pass());
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_bind_descriptor_sets(
                    self.command_buffer.get(ctx.frame()),
                    vk::PipelineBindPoint::GRAPHICS,
                    *instance.cast::<VkShaderInstance>().pipeline_layout,
                    0,
                    &[instance
                        .cast::<VkShaderInstance>()
                        .descriptor_sets
                        .read()
                        .unwrap()
                        .get(ctx.frame())],
                    &[],
                );
        }
    }

    fn draw_mesh(&self, ctx: &CommandCtx, _mesh: &Arc<Mesh>, _instance_count: u32, _first_instance: u32) {
        todo!()
    }

    fn draw_mesh_advanced(
        &self, ctx: &CommandCtx,
        mesh: &Arc<Mesh>,
        first_index: u32,
        vertex_offset: i32,
        index_count: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        let index_buffer = mesh.index_buffer().cast::<VkBuffer>();
        let vertex_buffer = mesh.vertex_buffer().cast::<VkBuffer>();
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_bind_index_buffer(
                    self.command_buffer.get(ctx.frame()),
                    index_buffer.get_handle(ctx.frame()),
                    0 as vk::DeviceSize,
                    match mesh.index_type() {
                        IndexBufferType::Uint16 => vk::IndexType::UINT16,
                        IndexBufferType::Uint32 => vk::IndexType::UINT32,
                    },
                );

            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_bind_vertex_buffers(
                    self.command_buffer.get(ctx.frame()),
                    0,
                    &[vertex_buffer.get_handle(ctx.frame())],
                    &[0],
                )
        }
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_draw_indexed(
                    self.command_buffer.get(ctx.frame()),
                    index_count,
                    instance_count,
                    first_index,
                    vertex_offset,
                    first_instance,
                )
        }
    }

    fn draw_mesh_indirect(&self, ctx: &CommandCtx, _mesh: &Arc<Mesh>) {
        todo!()
    }

    fn draw_procedural(
        &self, ctx: &CommandCtx,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        unsafe {
            GfxVulkan::get().device.assume_init_ref().handle.cmd_draw(
                self.command_buffer.get(ctx.frame()),
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
    }

    fn set_scissor(&self, ctx: &CommandCtx, scissors: Scissors) {
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_set_scissor(
                    self.command_buffer.get(ctx.frame()),
                    0,
                    &[vk::Rect2D {
                        extent: vk::Extent2D {
                            width: scissors.width,
                            height: scissors.height,
                        },
                        offset: vk::Offset2D {
                            x: scissors.min_x,
                            y: scissors.min_y,
                        },
                    }],
                )
        }
    }

    fn push_constant(
        &self, ctx: &CommandCtx,
        program: &Arc<dyn ShaderProgram>,
        data: &BufferMemory,
        stage: ShaderStage,
    ) {
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_push_constants(
                    self.command_buffer.get(ctx.frame()),
                    *program.cast::<VkShaderProgram>().pipeline_layout,
                    match stage {
                        ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
                        ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
                        ShaderStage::TesselationEvaluate => vk::ShaderStageFlags::TESSELLATION_EVALUATION,
                        ShaderStage::TesselationControl => vk::ShaderStageFlags::TESSELLATION_CONTROL,
                        ShaderStage::Geometry => { vk::ShaderStageFlags::GEOMETRY }
                        ShaderStage::Compute => { vk::ShaderStageFlags::COMPUTE }
                    },
                    0,
                    data.as_slice(),
                )
        }
    }
}
