use std::sync::Arc;
use crate::{GfxBuffer, PassID, ShaderProgram};
use crate::surface::GfxImageID;

pub trait GfxCommandBuffer {
    fn bind_program(&self, image: &GfxImageID, program: Arc<dyn ShaderProgram>);
    fn draw_mesh(&self, image: &GfxImageID, mesh: Arc<dyn GfxBuffer>, instance_count: u32, first_instance: u32);
    fn draw_mesh_advanced(&self, image: &GfxImageID, mesh: Arc<dyn GfxBuffer>, first_index: u32, vertex_offset: u32, index_count: u32, instance_count: u32, first_instance: u32);
    fn draw_mesh_indirect(&self, image: &GfxImageID, mesh: Arc<dyn GfxBuffer>);
    fn draw_procedural(&self, image: &GfxImageID, vertex_count: u32, first_vertex: u32, instance_count: u32, first_instance: u32);
    fn set_scissor(&self);
    fn push_constant(&self);
    fn get_pass_id(&self) -> PassID;
}