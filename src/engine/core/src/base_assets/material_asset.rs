use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use gfx::shader::{PassID, ShaderProgram, ShaderProgramInfos, ShaderProgramStage, ShaderStage};
use shader_compiler::backends::backend_shaderc::{BackendShaderC, ShaderCIncluder};
use shader_compiler::CompilerBackend;
use shader_compiler::parser::Parser;
use shader_compiler::types::{InterstageData, ShaderLanguage};

use crate::asset::{AssetFactory, AssetMetaData, GameAsset};
use crate::asset_manager::AssetManager;
use crate::asset_type_id::AssetTypeID;
use crate::base_assets::material_instance_asset::MaterialInstanceAsset;

pub struct ShaderPermutation {
    pub shader: Arc<dyn ShaderProgram>,
}

pub struct MaterialAsset {
    virtual_path: RwLock<String>,
    meta_data: AssetMetaData,
    parsed_shader: RwLock<Option<Parser>>,
    permutations: RwLock<HashMap<PassID, ShaderPermutation>>,
    shader_backend: Box<dyn CompilerBackend>,
}

impl MaterialAsset {
    pub fn new(asset_manager: &Arc<AssetManager>) -> Arc<Self> {
        Arc::new(Self {
            meta_data: AssetMetaData::new(asset_manager),
            virtual_path: RwLock::default(),
            parsed_shader: RwLock::default(),
            permutations: RwLock::default(),
            shader_backend: Box::new(BackendShaderC::new()),
        })
    }
    
    pub fn instantiate(&self) -> Arc<MaterialInstanceAsset> {
        let instance = MaterialInstanceAsset::new(&self.meta_data.asset_manager);
        
        instance
    }

    pub fn set_shader_code(&self, virtual_path: &Path, shader_text: String) {
        let virtual_path = virtual_path.to_str().unwrap().to_string();
        *self.virtual_path.write().unwrap() = virtual_path.clone();

        let parse_result = Parser::new(&shader_text, &virtual_path, Box::new(ShaderCIncluder::new()));
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

    pub fn get_program(&self, pass: &PassID) -> Option<Arc<dyn ShaderProgram>> {
        match self.permutations.read().unwrap().get(pass) {
            None => {}
            Some(permutation) => {
                return Some(permutation.shader.clone());
            }
        }

        match &*self.parsed_shader.read().unwrap() {
            None => {}
            Some(parser) => {
                // Vertex shader
                let vertex_code = match parser.program_data.get_data(&pass, &ShaderStage::Vertex) {
                    Ok(code) => { code }
                    Err(error) => {
                        println!("failed to get vertex shader data :\n{}", error.to_string());
                        return None;
                    }
                };
                // Fragment shader
                let fragment_code = match parser.program_data.get_data(&pass, &ShaderStage::Fragment) {
                    Ok(code) => { code }
                    Err(error) => {
                        println!("failed to get fragment shader data :\n{}", error.to_string());
                        return None;
                    }
                };
                
                let vertex_sprv = match self.shader_backend.compile_to_spirv(vertex_code, Path::new(self.virtual_path.read().unwrap().as_str()), ShaderLanguage::HLSL, ShaderStage::Vertex, InterstageData {
                    stage_outputs: Default::default(),
                    binding_index: 0,
                }) {
                    Ok(sprv) => { sprv }
                    Err(error) => {
                        println!("Failed to compile vertex shader : \n{}", error.to_string());
                        return None;
                    }
                };

                let fragment_sprv = match self.shader_backend.compile_to_spirv(fragment_code, Path::new(self.virtual_path.read().unwrap().as_str()), ShaderLanguage::HLSL, ShaderStage::Fragment, InterstageData {
                    stage_outputs: Default::default(),
                    binding_index: 0,
                }) {
                    Ok(sprv) => { sprv }
                    Err(error) => {
                        println!("Failed to compile fragment shader : \n{}", error.to_string());
                        return None;
                    }
                };
                let ci_shader = ShaderProgramInfos {
                    vertex_stage: ShaderProgramStage {
                        spirv: vertex_sprv.binary,
                        descriptor_bindings: vertex_sprv.bindings,
                        push_constant_size: 0,
                        stage_input: vec![],
                    },
                    fragment_stage: ShaderProgramStage {
                        spirv: fragment_sprv.binary,
                        descriptor_bindings: fragment_sprv.bindings,
                        push_constant_size: 0,
                        stage_input: vec![],
                    },
                    culling: Default::default(),
                    front_face: Default::default(),
                    topology: Default::default(),
                    polygon_mode: Default::default(),
                    alpha_mode: Default::default(),
                    depth_test: false,
                    line_width: 1.0,
                };

                let render_pass = match self.meta_data.asset_manager.graphics().find_render_pass(pass) {
                    None => { panic!("failed to find render pass [{pass}]") }
                    Some(pass) => { pass }
                };

                let program = self.meta_data.asset_manager.graphics().create_shader_program(&render_pass, &ci_shader);
                self.permutations.write().unwrap().insert(pass.clone(), ShaderPermutation {
                    shader: program.clone()
                });

                return Some(program);
            }
        }
        None
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
        &self.meta_data
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