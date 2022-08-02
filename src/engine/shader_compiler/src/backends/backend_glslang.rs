use std::path::Path;

use crate::{InterstageData, ShaderBlock, ShaderCompiler, ShaderLanguage, ShaderStage};
use crate::includer::Includer;
use crate::types::ShaderErrorResult;

struct GlslangCompiler {}

impl GlslangCompiler {
    pub fn new(language: ShaderLanguage) -> Self {
        Self {}
    }
}

impl ShaderCompiler for GlslangCompiler {
    fn compile_to_spirv(shader_code: Vec<ShaderBlock>, source_language: ShaderLanguage, shader_stage: ShaderStage, previous_stage_data: InterstageData) -> Result<Vec<u32>, ShaderErrorResult> {
        let errors = ShaderErrorResult::default();
        
        Err(errors)
    }
}

pub struct GlslangIncluder {}

impl GlslangIncluder {
    pub fn new() -> Self {
        Self {}
    }
}

impl Includer for GlslangIncluder {
    fn include_local(&self, file: &String, shader_path: &Path) -> Result<(String, String), ShaderErrorResult> {
        let mut errors = ShaderErrorResult::default();
        errors.push(-1, -1, format!("failed to include '{}' : include are not supported yet", file).as_str(), shader_path.to_str().unwrap());
        Err(errors)
    }

    fn include_system(&self, file: &String, shader_path: &Path) -> Result<(String, String), ShaderErrorResult> {
        todo!()
    }

    fn release_include(&self, file: &String, shader_path: &Path) {}

    fn add_include_path(&self, include_path: &Path) {}
}