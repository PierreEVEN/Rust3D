use std::sync::Arc;
use crate::{BufferCreateInfo, GfxBuffer, GfxRef};
use crate::buffer::{BufferAccess, BufferType, BufferUsage};
use crate::surface::GfxImageID;

pub struct Mesh {
    index_buffer: Arc<dyn GfxBuffer>,
    vertex_buffer: Arc<dyn GfxBuffer>,
    index_buffer_type: IndexBufferType,
    vertex_struct_size: u32,
}

#[derive(Copy, Clone)]
pub enum IndexBufferType {
    Uint16 = 2,
    Uint32 = 4,
}

pub struct MeshCreateInfos {
    pub vertex_structure_size: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    pub buffer_type: BufferType,
    pub index_buffer_type: IndexBufferType,
    pub vertex_data: Option<Vec<u8>>,
    pub index_data: Option<Vec<u8>>,
}

impl Mesh {
    pub fn new(gfx: &GfxRef, name: String, create_infos: &MeshCreateInfos) -> Arc<Self> {
        let index_buffer = gfx.create_buffer(format!("mesh::{}::index", name), &BufferCreateInfo {
            buffer_type: create_infos.buffer_type,
            usage: BufferUsage::IndexData,
            access: BufferAccess::CpuToGpu,
            size: create_infos.index_count * create_infos.index_buffer_type as u32,
        });
        let vertex_buffer = gfx.create_buffer(format!("mesh::{}::vertex", name), &BufferCreateInfo {
            buffer_type: create_infos.buffer_type,
            usage: BufferUsage::VertexData,
            access: BufferAccess::CpuToGpu,
            size: create_infos.vertex_count * create_infos.vertex_structure_size,
        });

        Arc::new(Self {
            index_buffer,
            vertex_buffer,
            index_buffer_type: create_infos.index_buffer_type,
            vertex_struct_size: create_infos.vertex_structure_size,
        })
    }

    pub fn index_buffer(&self) -> &Arc<dyn GfxBuffer> {
        &self.index_buffer
    }

    pub fn vertex_buffer(&self) -> &Arc<dyn GfxBuffer> {
        &self.vertex_buffer
    }

    pub fn resize(&self, _: &GfxImageID, vertex_count: u32, index_count: u32) {
        self.index_buffer.resize_buffer(index_count * self.index_buffer_type as u32);
        self.vertex_buffer.resize_buffer(vertex_count * self.vertex_struct_size);
    }

    pub fn set_data(&self, image_id: &GfxImageID, from_vertex: u32, vertex_data: &[u8], from_index: u32, index_data: &[u8]) {
        self.index_buffer.set_data(image_id, from_index * self.index_buffer_type as u32, index_data);
        self.vertex_buffer.set_data(image_id, from_vertex * self.vertex_struct_size, vertex_data);
    }

    pub fn index_type(&self) -> IndexBufferType {
        self.index_buffer_type
    }
}