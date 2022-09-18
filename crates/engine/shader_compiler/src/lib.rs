use std::path::Path;
use gfx::shader::{DescriptorBinding, ShaderLanguage, ShaderStage};
use crate::parser::ShaderChunk;
use crate::types::{InterstageData, ShaderErrorResult};

pub mod parser;
mod file_iterator;
mod includer;
pub mod types;
mod reflect;

pub mod backends {
    pub mod backend_shaderc;
}

pub struct CompilationResult {
    pub binary: Vec<u32>,
    pub bindings: Vec<DescriptorBinding>,
    pub push_constant_size: u32,
}

pub trait CompilerBackend {
    fn compile_to_spirv(&self, shader_code: &Vec<ShaderChunk>, virtual_path: &Path, source_language: ShaderLanguage, shader_stage: ShaderStage, previous_stage_data: InterstageData) -> Result<CompilationResult, ShaderErrorResult>;
}