use gfx::shader::ShaderStage;
use crate::parser::ShaderChunk;
use crate::types::{InterstageData, ShaderErrorResult, ShaderLanguage};

pub mod parser;
mod file_iterator;
mod includer;
pub mod types;

pub mod backends {
    pub mod backend_shaderc;
}

pub struct CompilationResult {
    pub binary: Vec<u32>
}

pub trait CompilerBackend {
    fn compile_to_spirv(&self, shader_code: &Vec<ShaderChunk>, source_language: ShaderLanguage, shader_stage: ShaderStage, previous_stage_data: InterstageData) -> Result<CompilationResult, ShaderErrorResult>;
}