use std::path::Path;

use shaderc::{CompileOptions, Compiler, EnvVersion, IncludeCallbackResult, IncludeType, SourceLanguage, SpirvVersion, TargetEnv};
use gfx::shader::ShaderStage;

use crate::{CompilationResult, CompilerBackend, InterstageData, ShaderChunk, ShaderLanguage};
use crate::includer::Includer;
use crate::types::ShaderErrorResult;

pub struct BackendShaderC {}

impl BackendShaderC {
    pub fn new() -> Self {
        Self {}
    }
}

fn include_callback(_name: &str, include_type: IncludeType, _source: &str, _include_depth: usize) -> IncludeCallbackResult {
    match include_type {
        IncludeType::Relative => {}
        IncludeType::Standard => {}
    }

    return Err("failed to include file".to_string());
}

impl CompilerBackend for BackendShaderC {
    fn compile_to_spirv(&self, shader_code: &Vec<ShaderChunk>, source_language: ShaderLanguage, _shader_stage: ShaderStage, _previous_stage_data: InterstageData) -> Result<CompilationResult, ShaderErrorResult> {
        let mut errors = ShaderErrorResult::default();

        let compiler = match Compiler::new() {
            None => {
                errors.push(-1, -1, "failed to create shaderc compiler", "");
                return Err(errors);
            }
            Some(compiler) => { compiler }
        };


        let mut compile_options = match CompileOptions::new() {
            None => {
                errors.push(-1, -1, "failed to create shaderc compile option", "");
                return Err(errors);
            }
            Some(compiler) => { compiler }
        };
        compile_options.set_include_callback(include_callback);
        compile_options.set_auto_bind_uniforms(true);
        compile_options.set_hlsl_io_mapping(true);
        compile_options.set_auto_map_locations(true);
        compile_options.set_auto_bind_uniforms(true);
        compile_options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_2 as u32);
        compile_options.set_target_spirv(SpirvVersion::V1_6);
        compile_options.set_source_language(match source_language {
            ShaderLanguage::HLSL => { SourceLanguage::HLSL }
            ShaderLanguage::GLSL => { SourceLanguage::GLSL }
        });
        let mut shader = String::new();
        for block in shader_code {
            shader += block.content.as_str();
        }

        let binary_result = match compiler.compile_into_spirv(&shader, shaderc::ShaderKind::Vertex, "shader.glsl", "main", Some(&compile_options)) {
            Ok(binary) => { binary }
            Err(compile_error) => {
                errors.push(-1, -1, format!("failed to compile shader to spirv : {}\n\n{}", compile_error.to_string(), shader).as_str(), "spirv shader binary code");
                return Err(errors);
            }
        };


        let binary_result = Vec::from(binary_result.as_binary());


        Ok(CompilationResult {
            binary: binary_result
        })
    }
}

pub struct ShaderCIncluder {}

impl ShaderCIncluder {
    pub fn new() -> Self {
        Self {}
    }
}

impl Includer for ShaderCIncluder {
    fn include_local(&self, file: &String, shader_path: &Path) -> Result<(String, String), ShaderErrorResult> {
        let mut errors = ShaderErrorResult::default();
        errors.push(-1, -1, format!("failed to include '{}' : include are not supported yet", file).as_str(), shader_path.to_str().unwrap());
        Err(errors)
    }

    fn include_system(&self, _file: &String, _shader_path: &Path) -> Result<(String, String), ShaderErrorResult> {
        todo!()
    }

    fn release_include(&self, _file: &String, _shader_path: &Path) {}

    fn add_include_path(&self, _include_path: &Path) {}
}