use std::path::Path;

use shaderc::{CompileOptions, Compiler, EnvVersion, IncludeCallbackResult, IncludeType, SourceLanguage, SpirvVersion, TargetEnv};

use gfx::shader::ShaderStage;

use crate::{CompilationResult, CompilerBackend, InterstageData, ShaderChunk, ShaderLanguage};
use crate::includer::Includer;
use crate::reflect::SpirvReflector;
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
    fn compile_to_spirv(&self, shader_code: &Vec<ShaderChunk>, virtual_path: &Path, source_language: ShaderLanguage, _shader_stage: ShaderStage, _previous_stage_data: InterstageData) -> Result<CompilationResult, ShaderErrorResult> {
        let mut errors = ShaderErrorResult::default();

        let compiler = match Compiler::new() {
            None => {
                errors.push(None, None, "BackendShaderC::compile_to_spirv", "failed to create shaderc compiler", virtual_path.to_str().unwrap());
                return Err(errors);
            }
            Some(compiler) => { compiler }
        };


        let mut compile_options = match CompileOptions::new() {
            None => {
                errors.push(None, None, "BackendShaderC::compile_to_spirv", "failed to create shaderc compile option", virtual_path.to_str().unwrap());
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
        compile_options.set_target_spirv(SpirvVersion::V1_5);
        compile_options.set_source_language(match source_language {
            ShaderLanguage::HLSL => { SourceLanguage::HLSL }
            ShaderLanguage::GLSL => { SourceLanguage::GLSL }
        });
        let mut shader = String::new();
        for block in shader_code {
            shader += block.content.as_str();
        }

        let binary_result = match compiler.compile_into_spirv(&shader, match _shader_stage {
            ShaderStage::Vertex => { shaderc::ShaderKind::Vertex }
            ShaderStage::Fragment => { shaderc::ShaderKind::Fragment }
        }, virtual_path.to_str().unwrap(), "main", Some(&compile_options)) {
            Ok(binary) => { binary }
            Err(compile_error) => {
                let mut error = compile_error.to_string();
                error += "\n";
                println!("test : {}", compile_error);
                for line in compile_error.to_string().split("\n") {
                    let mut line_pos = None;
                    let mut column = None;

                    let message = if line.contains(": error:") {
                        line_pos = Some(line.split(":").nth(1).unwrap().parse::<isize>().unwrap());
                        line.split(": error:").nth(1).unwrap()
                    } else if line.contains("): error") {
                        let error = line.split("): error").nth(1).unwrap();
                        if error.contains("at column ") {
                            line_pos = Some(line.split("(").nth(1).unwrap().split(")").nth(0).unwrap().parse::<isize>().unwrap());
                            column = Some(error.split("at column ").nth(1).unwrap().split(", ").nth(0).unwrap().parse::<isize>().unwrap());
                            error.split(", ").nth(1).unwrap()
                        } else {
                            error
                        }
                    } else if line.contains("compilation error") || !line.chars().any(|c| c.is_ascii_alphanumeric()) {
                        continue;
                    } else {
                        line
                    };


                    if line_pos.is_some() {
                        let line = line_pos.unwrap();
                        for i in 0..shader_code.len() {
                            if shader_code[i].line_start > line as u32 && i > 0 {
                                line_pos = Some(line + shader_code[i - 1].line_start as isize);
                                break;
                            }
                        }
                    }

                    errors.push(line_pos, column, "BackendShaderC::compile_to_spirv", message, virtual_path.to_str().unwrap());
                }

                errors.push(None, None, "Shader compilation failed", format!("{shader}").as_str(), virtual_path.to_str().unwrap());

                return Err(errors);
            }
        };
        let binary_result = Vec::from(binary_result.as_binary());

        let reflector = SpirvReflector::new(&binary_result);


        Ok(CompilationResult {
            binary: binary_result,
            bindings: reflector.bindings,
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
    fn include_local(&self, file: &String, _virtual_path: &String) -> Result<(String, String), ShaderErrorResult> {
        let mut errors = ShaderErrorResult::default();
        errors.push(None, None, "ShaderCIncluder::include_local", format!("failed to include '{}' : include are not supported yet", file).as_str(), _virtual_path);
        Err(errors)
    }

    fn include_system(&self, _file: &String, _virtual_path: &String) -> Result<(String, String), ShaderErrorResult> {
        todo!()
    }

    fn release_include(&self, _file: &String, _virtual_path: &String) {}

    fn add_include_path(&self, _include_path: &String) {}
}