use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use gfx::shader::{PassID, ShaderProgram};
use shader_compiler::backends::backend_shaderc::ShaderCIncluder;
use shader_compiler::parser::Parser;

use crate::asset::{AssetFactory, AssetMetaData, GameAsset};
use crate::asset_manager::AssetManager;
use crate::asset_type_id::AssetTypeID;

pub struct ShaderPermutation {
    pub pass_id: PassID,
    pub code: Vec<u32>,
    pub shader: Arc<dyn ShaderProgram>,
}

pub struct MaterialAsset {
    virtual_path: RwLock<String>,
    _meta_data: AssetMetaData,
    parsed_shader: RwLock<Option<Parser>>,
    permutations: RwLock<HashMap<PassID, ShaderPermutation>>,
}

impl MaterialAsset {
    pub fn new(asset_manager: &Arc<AssetManager>) -> Arc<MaterialAsset> {
        Arc::new(Self {
            _meta_data: AssetMetaData::new(asset_manager),
            virtual_path: RwLock::default(),
            parsed_shader: RwLock::default(),
            permutations: RwLock::default(),
        })
    }

    pub fn set_shader_code(&self, shader_text: String) {
        let virtual_path = &*self.virtual_path.read().unwrap();
        let parse_result = Parser::new(&shader_text, virtual_path, Box::new(ShaderCIncluder::new()));
        match parse_result {
            Ok(result) => {
                *self.parsed_shader.write().unwrap() = Some(result)
            }
            Err(error) => {
                *self.parsed_shader.write().unwrap() = None;
                panic!("shader syntax error : \n{}", error.to_string())
            }
        };
        self.permutations.write().unwrap().clear();
    }

    /*
    pub fn text_to_sprv(text_code: String, virtual_path: &Path) -> Vec<u32> {
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
    */

    pub fn get_program(&self, pass: &PassID) -> Option<Arc<dyn ShaderProgram>> {
        match self.permutations.read().unwrap().get(pass) {
            None => {
                None
            }
            Some(permutation) => {
                Some(permutation.shader.clone())
            }
        }
    }
}

impl GameAsset for MaterialAsset {
    fn save(&self) -> Result<(), String> {
        todo!()
    }

    fn reload(&self) -> Result<(), String> {
        todo!()
    }

    fn meta_data(&self) -> &AssetMetaData {
        &self._meta_data
    }
}

pub struct MaterialAssetFactory {}

impl MaterialAssetFactory {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}


impl AssetFactory for MaterialAssetFactory {
    fn instantiate_from_asset_path(&self, _path: &Path) -> Arc<dyn GameAsset> {
        todo!()
    }

    fn asset_id(&self) -> AssetTypeID {
        AssetTypeID::from("material")
    }
}