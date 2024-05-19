use std::sync::Arc;
use maths::vec2::Vec2u32;

use shader_base::pass_id::PassID;
use shader_base::ShaderStage;
use shader_base::types::{GfxCast, Scissors};

use crate::gfx::buffer::BufferMemory;
use crate::gfx::mesh::Mesh;
use crate::gfx::shader::ShaderProgram;
use crate::gfx::shader_instance::ShaderInstance;
use crate::gfx::surface::Frame;

pub struct CommandCtx {
    frame: Frame,
    render_pass: PassID,
    display_res: Vec2u32,
}

impl CommandCtx {
    pub fn new(frame: Frame, render_pass: PassID, display_res: Vec2u32) -> Self {
        Self { frame, render_pass, display_res }
    }
    pub fn frame(&self) -> &Frame { &self.frame }
    pub fn render_pass(&self) -> &PassID { &self.render_pass }
    pub fn display_res(&self) -> &Vec2u32 { &self.display_res }
}

pub trait GfxCommandBuffer: GfxCast {
    fn bind_program(&self, ctx: &CommandCtx, program: &Arc<dyn ShaderProgram>);
    fn bind_shader_instance(&self, ctx: &CommandCtx, shader_instance: &Arc<dyn ShaderInstance>);
    fn draw_mesh(&self, ctx: &CommandCtx, mesh: &Arc<Mesh>, instance_count: u32, first_instance: u32);
    fn draw_mesh_advanced(
        &self, 
        ctx: &CommandCtx,
        mesh: &Arc<Mesh>,
        first_index: u32,
        vertex_offset: i32,
        index_count: u32,
        instance_count: u32,
        first_instance: u32,
    );
    fn draw_mesh_indirect(&self, ctx: &CommandCtx, mesh: &Arc<Mesh>);
    fn draw_procedural(
        &self, 
        ctx: &CommandCtx,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    );
    fn set_scissor(&self, ctx: &CommandCtx, scissors: Scissors);
    fn push_constant(
        &self, 
        ctx: &CommandCtx,
        program: &Arc<dyn ShaderProgram>,
        data: &BufferMemory,
        stage: ShaderStage,
    );
}

impl dyn GfxCommandBuffer {
    pub fn cast<U: GfxCommandBuffer + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}
