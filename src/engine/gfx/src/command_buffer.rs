use std::sync::Arc;
use crate::{GfxBuffer, GfxCast, PassID, ShaderInstance, ShaderProgram};

pub trait GfxCommandBuffer : GfxCast {
    fn bind_program(&self, program: &Arc<dyn ShaderProgram>);
    fn bind_shader_instance(&self, shader_instance: &Arc<dyn ShaderInstance>);
    fn draw_mesh(&self, mesh: &Arc<dyn GfxBuffer>, instance_count: u32, first_instance: u32);
    fn draw_mesh_advanced(&self, mesh: &Arc<dyn GfxBuffer>, first_index: u32, vertex_offset: u32, index_count: u32, instance_count: u32, first_instance: u32);
    fn draw_mesh_indirect(&self, mesh: &Arc<dyn GfxBuffer>);
    fn draw_procedural(&self, vertex_count: u32, first_vertex: u32, instance_count: u32, first_instance: u32);
    fn set_scissor(&self);
    fn push_constant(&self);
    fn get_pass_id(&self) -> PassID;
}

impl dyn GfxCommandBuffer {
    pub fn cast<U: GfxCommandBuffer + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}