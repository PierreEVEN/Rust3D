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

    fn set_data(&self, from_vertex: u32, vertex_data: &[u8], from_index: u32, index_data: &[u8]) {
        if (from_index + index_data.len() as u32) * self.index_buffer_type as u32 > self.index_buffer.buffer_size() {
            self.index_buffer.resize_buffer((from_index + index_data.len() as u32) * self.index_buffer_type as u32)
        }

        if (from_vertex + vertex_data.len() as u32) * self.vertex_struct_size as u32 > self.vertex_buffer.buffer_size()  {
            self.vertex_buffer.resize_buffer((from_vertex + vertex_data.len() as u32) * self.vertex_struct_size as u32)
        }
        
        let index_memory = self.index_buffer.get_buffer_memory();
        unsafe { index_data.as_ptr().copy_to(index_memory.get_ptr(self.index_buffer_type as usize * from_index as usize), index_data.len() as usize * self.index_buffer_type as usize) }
        
        let vertex_memory = self.vertex_buffer.get_buffer_memory();
        unsafe { vertex_data.as_ptr().copy_to(vertex_memory.get_ptr(self.vertex_struct_size as usize * from_vertex as usize), vertex_data.len() as usize * self.vertex_struct_size as usize) }
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