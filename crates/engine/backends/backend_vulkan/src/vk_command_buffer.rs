use std::sync::{Arc, RwLock};

use ash::vk;

use gfx::buffer::{BufferMemory};
use gfx::command_buffer::GfxCommandBuffer;
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::mesh::{IndexBufferType, Mesh};
use gfx::shader::{PassID, ShaderProgram, ShaderStage};
use gfx::shader_instance::ShaderInstance;
use gfx::surface::{GfxImageID, GfxSurface};
use gfx::types::Scissors;

use crate::{GfxVulkan, vk_check, VkBuffer, VkShaderInstance, VkShaderProgram};

pub struct VkCommandPool {
    pub command_pool: vk::CommandPool,
}

pub fn create_command_buffer(gfx: &GfxRef, name: String) -> vk::CommandBuffer {
    let create_infos = vk::CommandBufferAllocateInfo::builder()
        .command_pool(gfx.cast::<GfxVulkan>().command_pool.command_pool)
        .command_buffer_count(1)
        .level(vk::CommandBufferLevel::PRIMARY)
        .build();
    gfx.cast::<GfxVulkan>().set_vk_object_name(
        vk_check!(unsafe { gfx.cast::<GfxVulkan>().device.handle.
            allocate_command_buffers(&create_infos) })[0],
        format!("<(command_buffer)> {}", name).as_str(),
    )
}

pub fn begin_command_buffer(gfx: &GfxRef, command_buffer: vk::CommandBuffer, one_time: bool) {
    vk_check!(unsafe { gfx.cast::<GfxVulkan>().device.handle.begin_command_buffer(command_buffer, 
        &vk::CommandBufferBeginInfo::builder()
        .flags(if one_time { vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT } else { vk::CommandBufferUsageFlags::empty() })
        .build())
    });
}

pub fn end_command_buffer(gfx: &GfxRef, command_buffer: vk::CommandBuffer) {
    vk_check!(unsafe { gfx.cast::<GfxVulkan>().device.handle.end_command_buffer(command_buffer)})
}

pub fn submit_command_buffer(gfx: &GfxRef, command_buffer: vk::CommandBuffer, queue_flags: vk::QueueFlags) {
    match gfx.cast::<GfxVulkan>().device.get_queue(queue_flags) {
        Ok(queue) => {
            queue.submit(vk::SubmitInfo::builder()
                .command_buffers(&[command_buffer])
                .build());
        }
        Err(_) => {
            panic!("failed to find queue");
        }
    }
}


impl VkCommandPool {
    pub fn new(gfx: &GfxRef, name: String) -> VkCommandPool {
        let create_infos = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(match gfx.cast::<GfxVulkan>().device.queues.get(&vk::QueueFlags::GRAPHICS) {
                None => { panic!("failed to find queue"); }
                Some(queue) => { queue[0].index }
            })
            .build();

        let command_pool = vk_check!(unsafe {gfx.cast::<GfxVulkan>().device.handle.create_command_pool(&create_infos, None)});

        gfx.cast::<GfxVulkan>().set_vk_object_name(command_pool, format!("<(command_pool)> {}", name).as_str());

        VkCommandPool {
            command_pool
        }
    }
}

pub struct VkCommandBuffer {
    pub command_buffer: GfxResource<vk::CommandBuffer>,
    gfx: GfxRef,
    pass_id: RwLock<PassID>,
    image_id: RwLock<GfxImageID>,
    surface: Arc<dyn GfxSurface>,
}

pub struct RbCommandBuffer {
    name: String,
}

impl GfxImageBuilder<vk::CommandBuffer> for RbCommandBuffer {
    fn build(&self, gfx: &GfxRef, _: &GfxImageID) -> vk::CommandBuffer {
        create_command_buffer(gfx, self.name.clone())
    }
}

impl VkCommandBuffer {
    pub fn new(gfx: &GfxRef, name: String, surface: &Arc<dyn GfxSurface>) -> Arc<VkCommandBuffer> {
        Arc::new(VkCommandBuffer {
            command_buffer: GfxResource::new(gfx, RbCommandBuffer { name }),
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
                vk::PipelineBindPoint::GRAPHICS,
                program.cast::<VkShaderProgram>().pipeline,
            );
        }
    }

    fn bind_shader_instance(&self, instance: &Arc<dyn ShaderInstance>) {
        instance.cast::<VkShaderInstance>().refresh_descriptors(&*self.image_id.read().unwrap());
        unsafe {
            self.gfx.cast::<GfxVulkan>().device.handle.cmd_bind_descriptor_sets(
                self.command_buffer.get(&*self.image_id.read().unwrap()),
                vk::PipelineBindPoint::GRAPHICS,
                *instance.cast::<VkShaderInstance>().pipeline_layout,
                0,
                &[instance.cast::<VkShaderInstance>().descriptor_sets.read().unwrap().get(&*self.image_id.read().unwrap())],
                &[],
            );
        }
    }

    fn draw_mesh(&self, _mesh: &Arc<Mesh>, _instance_count: u32, _first_instance: u32) {
        todo!()
    }

    fn draw_mesh_advanced(&self, mesh: &Arc<Mesh>, first_index: u32, vertex_offset: i32, index_count: u32, instance_count: u32, first_instance: u32) {
        let index_buffer = mesh.index_buffer().cast::<VkBuffer>();
        let vertex_buffer = mesh.vertex_buffer().cast::<VkBuffer>();
        unsafe {
            self.gfx.cast::<GfxVulkan>().device.handle.cmd_bind_index_buffer(
                self.command_buffer.get(&*self.image_id.read().unwrap()),
                index_buffer.get_handle(&*self.image_id.read().unwrap()),
                0 as vk::DeviceSize,
                match mesh.index_type() {
                    IndexBufferType::Uint16 => { vk::IndexType::UINT16 }
                    IndexBufferType::Uint32 => { vk::IndexType::UINT32 }
                });

            self.gfx.cast::<GfxVulkan>().device.handle.cmd_bind_vertex_buffers(
                self.command_buffer.get(&*self.image_id.read().unwrap()),
                0,
                &[vertex_buffer.get_handle(&*self.image_id.read().unwrap())],
                &[0])
        }
        unsafe { self.gfx.cast::<GfxVulkan>().device.handle.cmd_draw_indexed(self.command_buffer.get(&*self.image_id.read().unwrap()), index_count, instance_count, first_index, vertex_offset, first_instance) }
    }

    fn draw_mesh_indirect(&self, _mesh: &Arc<Mesh>) {
        todo!()
    }

    fn draw_procedural(&self, vertex_count: u32, first_vertex: u32, instance_count: u32, first_instance: u32) {
        unsafe { self.gfx.cast::<GfxVulkan>().device.handle.cmd_draw(self.command_buffer.get(&*self.image_id.read().unwrap()), vertex_count, instance_count, first_vertex, first_instance) }
    }

    fn set_scissor(&self, scissors: Scissors) {
        unsafe {
            self.gfx.cast::<GfxVulkan>().device.handle.cmd_set_scissor(self.command_buffer.get(&*self.image_id.read().unwrap()), 0, &[vk::Rect2D {
                extent: vk::Extent2D { width: scissors.width, height: scissors.height },
                offset: vk::Offset2D { x: scissors.min_x, y: scissors.min_y },
            }])
        }
    }

    fn push_constant(&self, program: &Arc<dyn ShaderProgram>, data: BufferMemory, stage: ShaderStage) {
        unsafe {
            self.gfx.cast::<GfxVulkan>().device.handle.cmd_push_constants(self.command_buffer.get(&*self.image_id.read().unwrap()), *program.cast::<VkShaderProgram>().pipeline_layout, match stage {
                ShaderStage::Vertex => { vk::ShaderStageFlags::VERTEX }
                ShaderStage::Fragment => { vk::ShaderStageFlags::FRAGMENT }
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