use std::sync::Arc;
use gfx::buffer::{BufferAccess, BufferCreateInfo, BufferUsage, GfxBuffer};
use gfx::GfxRef;
use gfx::mesh::{IndexBufferType, Mesh, MeshCreateInfos};

pub struct VkMesh {
    index_buffer: Arc<dyn GfxBuffer>,
    vertex_buffer: Arc<dyn GfxBuffer>,
    index_buffer_type: IndexBufferType,
    vertex_struct_size: u32,
}

impl Mesh for VkMesh {
    fn index_buffer(&self) -> &Arc<dyn GfxBuffer> {
        &self.index_buffer
    }

    fn vertex_buffer(&self) -> &Arc<dyn GfxBuffer> {
        &self.vertex_buffer
    }

    fn set_data(&self, index_count: u32, vertex_count: u32, vertex_data: &[u8], index_data: &[u8]) {
        
        if self.index_buffer.buffer_size() != index_count * self.index_buffer_type as u32 {
            self.index_buffer.resize_buffer()
        }

        if self.vertex_buffer.buffer_size() != index_count * self.vertex_struct_size as u32 {
            self.vertex_buffer.resize_buffer()
        }
        
    }
}

impl VkMesh {
    pub fn new(gfx: &GfxRef, create_infos: &MeshCreateInfos) -> Arc<Self> {

        let index_buffer = gfx.create_buffer(&BufferCreateInfo {
            buffer_type: create_infos.buffer_type,
            usage: BufferUsage::IndexData,
            access: BufferAccess::CpuToGpu,
            size: create_infos.index_count * create_infos.index_buffer_type as u32,
        });
        let vertex_buffer = gfx.create_buffer(&BufferCreateInfo {
            buffer_type: create_infos.buffer_type,
            usage: BufferUsage::VertexData,
            access: BufferAccess::CpuToGpu,
            size: create_infos.vertex_count * create_infos.vertex_structure_size,
        });
        
        Arc::new(Self {
            index_buffer,
            vertex_buffer,
            index_buffer_type: create_infos.index_buffer_type,
            vertex_struct_size: create_infos.vertex_structure_size
        })
    }
}