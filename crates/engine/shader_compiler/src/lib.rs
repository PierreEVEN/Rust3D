use crate::parser::ShaderChunk;
use crate::types::{InterstageData, ShaderErrorResult};
use gfx::shader::{DescriptorBinding};
use std::path::Path;
use shader_base::{ShaderLanguage, ShaderStage};

mod file_iterator;
mod includer;
pub mod parser;
mod reflect;
pub mod types;

pub mod backends {
    pub mod backend_shaderc;
}

pub struct CompilationResult {
    pub binary: Vec<u32>,
    pub bindings: Vec<DescriptorBinding>,
    pub push_constant_size: u32,
}

pub trait CompilerBackend {
    fn compile_to_spirv(
        &self,
        shader_code: &[ShaderChunk],
        virtual_path: &Path,
        source_language: ShaderLanguage,
        shader_stage: ShaderStage,
        previous_stage_data: InterstageData,
    ) -> Result<CompilationResult, ShaderErrorResult>;
}
