﻿
use std::sync::Arc;
use crate::buffer::BufferType;
use crate::{GfxBuffer, GfxCast};
use crate::surface::GfxImageID;

#[derive(Copy, Clone)]
pub enum IndexBufferType {
    Uint16 = 2,
    Uint32 = 4,
}

pub struct MeshVertexData {}

pub struct MeshCreateInfos {
    pub vertex_structure_size: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    pub buffer_type: BufferType,
    pub index_buffer_type: IndexBufferType,
    pub vertex_data: Option<Vec<u8>>,
    pub index_data: Option<Vec<u8>>,
}

pub trait Mesh: GfxCast {
    fn index_buffer(&self) -> &Arc<dyn GfxBuffer>;
    fn vertex_buffer(&self) -> &Arc<dyn GfxBuffer>;
    fn set_data(&self, image_id: &GfxImageID, from_vertex: u32, vertex_data: &[u8], from_index: u32, index_data: &[u8]);
    fn index_type(&self) -> IndexBufferType;
}

impl dyn Mesh {
    pub fn cast<U: Mesh + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}