use crate::buffer::BufferMemory;
use crate::{Mesh, PassID, ShaderInstance, ShaderProgram};
use std::sync::Arc;
use maths::vec2::Vec2u32;
use shader_base::ShaderStage;
use shader_base::types::{GfxCast, Scissors};
use crate::surface::Frame;

pub trait GfxCommandBuffer: GfxCast {
    fn bind_program(&self, program: &Arc<dyn ShaderProgram>);
    fn bind_shader_instance(&self, shader_instance: &Arc<dyn ShaderInstance>);
    fn draw_mesh(&self, mesh: &Arc<Mesh>, instance_count: u32, first_instance: u32);
    fn draw_mesh_advanced(
        &self,
        mesh: &Arc<Mesh>,
        first_index: u32,
        vertex_offset: i32,
        index_count: u32,
        instance_count: u32,
        first_instance: u32,
    );
    fn draw_mesh_indirect(&self, mesh: &Arc<Mesh>);
    fn draw_procedural(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    );
    fn set_scissor(&self, scissors: Scissors);
    fn push_constant(
        &self,
        program: &Arc<dyn ShaderProgram>,
        data: &BufferMemory,
        stage: ShaderStage,
    );
    fn get_pass_id(&self) -> PassID;
    fn get_frame_id(&self) -> Frame;
    fn get_display_res(&self) -> Vec2u32;
}

impl dyn GfxCommandBuffer {
    pub fn cast<U: GfxCommandBuffer + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}
