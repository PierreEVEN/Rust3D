use std::sync::{Arc, RwLock};

use ash::vk;

use gfx::buffer::BufferMemory;
use gfx::command_buffer::GfxCommandBuffer;
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::mesh::{IndexBufferType, Mesh};
use gfx::shader::{ShaderProgram};
use gfx::shader_instance::ShaderInstance;
use gfx::surface::Frame;
use gfx::types::Scissors;
use shader_base::pass_id::PassID;
use shader_base::ShaderStage;

use crate::{vk_check, GfxVulkan, VkBuffer, VkShaderInstance, VkShaderProgram};

pub struct VkCommandPool {
    pub command_pool: vk::CommandPool,
}

pub fn create_command_buffer(name: String) -> vk::CommandBuffer {
    let create_infos = vk::CommandBufferAllocateInfo::builder()
        .command_pool(unsafe { GfxVulkan::get().command_pool.assume_init_ref() }.command_pool)
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
    pub fn new(name: String) -> VkCommandPool {
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
            .set_vk_object_name(command_pool, format!("command pool\t\t: {}", name).as_str());

        VkCommandPool { command_pool }
    }
}

pub struct VkCommandBuffer {
    pub command_buffer: GfxResource<vk::CommandBuffer>,
    pass_id: RwLock<PassID>,
    image_id: RwLock<Frame>,
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
            command_buffer: GfxResource::new(RbCommandBuffer { name }),
            pass_id: RwLock::new(PassID::new("undefined")),
            image_id: RwLock::new(Frame::null()),
        })
    }

    pub fn init_for(&self, new_id: PassID, image_id: Frame) {
        *self.pass_id.write().unwrap() = new_id;
        *self.image_id.write().unwrap() = image_id;
    }
}

impl GfxCommandBuffer for VkCommandBuffer {
    fn bind_program(&self, program: &Arc<dyn ShaderProgram>) {
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_bind_pipeline(
                    self.command_buffer.get(&self.image_id.read().unwrap()),
                    vk::PipelineBindPoint::GRAPHICS,
                    program.cast::<VkShaderProgram>().pipeline,
                );
        }
    }

    fn bind_shader_instance(&self, instance: &Arc<dyn ShaderInstance>) {
        instance
            .cast::<VkShaderInstance>()
            .refresh_descriptors(&self.image_id.read().unwrap());
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_bind_descriptor_sets(
                    self.command_buffer.get(&self.image_id.read().unwrap()),
                    vk::PipelineBindPoint::GRAPHICS,
                    *instance.cast::<VkShaderInstance>().pipeline_layout,
                    0,
                    &[instance
                        .cast::<VkShaderInstance>()
                        .descriptor_sets
                        .read()
                        .unwrap()
                        .get(&self.image_id.read().unwrap())],
                    &[],
                );
        }
    }

    fn draw_mesh(&self, _mesh: &Arc<Mesh>, _instance_count: u32, _first_instance: u32) {
        todo!()
    }

    fn draw_mesh_advanced(
        &self,
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
                    self.command_buffer.get(&self.image_id.read().unwrap()),
                    index_buffer.get_handle(&self.image_id.read().unwrap()),
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
                    self.command_buffer.get(&self.image_id.read().unwrap()),
                    0,
                    &[vertex_buffer.get_handle(&self.image_id.read().unwrap())],
                    &[0],
                )
        }
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_draw_indexed(
                    self.command_buffer.get(&self.image_id.read().unwrap()),
                    index_count,
                    instance_count,
                    first_index,
                    vertex_offset,
                    first_instance,
                )
        }
    }

    fn draw_mesh_indirect(&self, _mesh: &Arc<Mesh>) {
        todo!()
    }

    fn draw_procedural(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        unsafe {
            GfxVulkan::get().device.assume_init_ref().handle.cmd_draw(
                self.command_buffer.get(&self.image_id.read().unwrap()),
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
    }

    fn set_scissor(&self, scissors: Scissors) {
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_set_scissor(
                    self.command_buffer.get(&self.image_id.read().unwrap()),
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
        &self,
        program: &Arc<dyn ShaderProgram>,
        data: BufferMemory,
        stage: ShaderStage,
    ) {
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_push_constants(
                    self.command_buffer.get(&self.image_id.read().unwrap()),
                    *program.cast::<VkShaderProgram>().pipeline_layout,
                    match stage {
                        ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
                        ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
                        ShaderStage::Tesselation => {vk::ShaderStageFlags::TESSELLATION_CONTROL}
                        ShaderStage::Geometry => {vk::ShaderStageFlags::GEOMETRY}
                        ShaderStage::Compute => {vk::ShaderStageFlags::COMPUTE}
                    },
                    0,
                    data.as_slice(),
                )
        }
    }

    fn get_pass_id(&self) -> PassID {
        self.pass_id.read().unwrap().clone()
    }
}
