use std::sync::Arc;
use crate::{GfxBuffer, ShaderProgram};
use crate::render_pass::RenderPassInstance;

pub trait GfxCommandBuffer {
    fn bind_program(&self, program: Arc<dyn ShaderProgram>);
    fn draw_mesh(&self, mesh: Arc<dyn GfxBuffer>, instance_count: u32, first_instance: u32);
    fn draw_mesh_advanced(&self, mesh: Arc<dyn GfxBuffer>, first_index: u32, vertex_offset: u32, index_count: u32, instance_count: u32, first_instance: u32);
    fn draw_mesh_indirect(&self, mesh: Arc<dyn GfxBuffer>);
    fn draw_procedural(&self, vertex_count: u32, first_vertex: u32, instance_count: u32, first_instance: u32);
    fn set_scissor(&self);
    fn push_constant(&self);
    
    fn init(&self);    
    fn submit(&self);
    
    fn begin_pass(&self, pass: &dyn RenderPassInstance);
    fn end_pass(&self, pass: &dyn RenderPassInstance);
}