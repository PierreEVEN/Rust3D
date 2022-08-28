use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};
use gfx::shader::ShaderStage;
use shader_compiler::backends::backend_shaderc::{BackendShaderC, ShaderCIncluder};
use shader_compiler::{CompilationResult, CompilerBackend};
use shader_compiler::parser::Parser;
use shader_compiler::types::{InterstageData, ShaderErrorResult, ShaderLanguage};

use crate::asset::GameAsset;

struct ShaderPermutation {
    
}

pub struct MaterialAsset {
    asset_name: RwLock<String>,
   // permutations: VkSwapchainResource<ShaderPermutation>
}

impl MaterialAsset {
    pub fn from_path(path: &Path) -> Arc<dyn GameAsset> {
        let mut errors = ShaderErrorResult::default();
        let shader_code = match fs::read_to_string(path) {
            Ok(file_data) => { file_data }
            Err(error) => {
                errors.push(-1, -1, &format!("failed to open file : {}", error), path.to_str().unwrap());
                "error : failed to open file".to_string()
            }
        };
        
        Self::from_sprv(Self::text_to_sprv(shader_code, path), "undefined".to_string())
    }
    
    pub fn from_sprv(code: Vec<u32>, name: String) -> Arc<dyn GameAsset> {
        Arc::new(Self {
            asset_name: RwLock::new(name)
        })
    }
    
    pub fn text_to_sprv(text_code: String, virtual_path: &Path) -> Vec<u32> {
        let includer = Box::new(ShaderCIncluder::new());
        let parse_result = match Parser::new(text_code, virtual_path, includer) {
            Ok(result) => {
                println!("successfully parsed shader");
                result
            }
            Err(error) => { panic!("shader syntax error : \n{}", error.to_string()) }
        };

        let shader_compiler = BackendShaderC::new();

        for pass in ["gbuffer".to_string()] {
            let interstage = InterstageData {
                stage_outputs: Default::default(),
                binding_index: 0,
            };

            let null_shader = Vec::new();
            let vertex_code = match parse_result.program_data.get_data(&pass, &ShaderStage::Vertex) {
                Ok(code) => { code }
                Err(error) => {
                    println!("failed to get vertex shader code : \n{}", error.to_string());
                    &null_shader
                }
            };

            let _sprv = match shader_compiler.compile_to_spirv(vertex_code, ShaderLanguage::HLSL, ShaderStage::Vertex, interstage) {
                Ok(sprv) => {
                    println!("compilation succeeded");
                    sprv
                }
                Err(error) => {
                    println!("shader compilation error : \n{}", error.to_string());
                    CompilationResult { binary: vec![] }
                }
            };
        }
        
        Vec::new()
    }
}

impl GameAsset for MaterialAsset {
    fn get_name(&self) -> String {
        self.asset_name.read().unwrap().clone()
    }
}