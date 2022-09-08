use std::sync::Arc;
use crate::{GfxCast, GfxSurface, Mesh, PassID, ShaderInstance, ShaderProgram};
use crate::buffer::BufferMemory;
use crate::shader::ShaderStage;
use crate::types::Scissors;

pub trait GfxCommandBuffer : GfxCast {
    fn bind_program(&self, program: &Arc<dyn ShaderProgram>);
    fn bind_shader_instance(&self, shader_instance: &Arc<dyn ShaderInstance>);
    fn draw_mesh(&self, mesh: &Arc<dyn Mesh>, instance_count: u32, first_instance: u32);
    fn draw_mesh_advanced(&self, mesh: &Arc<dyn Mesh>, first_index: u32, vertex_offset: i32, index_count: u32, instance_count: u32, first_instance: u32);
    fn draw_mesh_indirect(&self, mesh: &Arc<dyn Mesh>);
    fn draw_procedural(&self, vertex_count: u32, first_vertex: u32, instance_count: u32, first_instance: u32);
    fn set_scissor(&self, scissors: Scissors);
    fn push_constant(&self, program: &Arc<dyn ShaderProgram>, data: BufferMemory, stage: ShaderStage);
    fn get_pass_id(&self) -> PassID;
    fn get_surface(&self) -> Arc<dyn GfxSurface>;
}

impl dyn GfxCommandBuffer {
    pub fn cast<U: GfxCommandBuffer + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}