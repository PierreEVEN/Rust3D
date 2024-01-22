use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use shader_base::pass_id::PassID;
use crate::base_assets::asset::{AssetFactory, AssetMetaData, GameAsset};
use crate::base_assets::asset_type_id::AssetTypeID;
use crate::base_assets::material_instance_asset::MaterialInstanceAsset;
use crate::gfx::shader::ShaderProgram;

pub struct ShaderPermutation {
    pub shader: Arc<dyn ShaderProgram>,
}

pub struct MaterialAsset {
    virtual_path: RwLock<String>,
    meta_data: AssetMetaData,
    permutations: RwLock<HashMap<PassID, ShaderPermutation>>,
}

impl MaterialAsset {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            meta_data: AssetMetaData::new(),
            virtual_path: RwLock::default(),
            permutations: RwLock::default(),
        })
    }

    pub fn instantiate(&self) -> Arc<MaterialInstanceAsset> {
        MaterialInstanceAsset::new()
    }

    pub fn set_shader_code(&self, virtual_path: &Path, _shader_text: String) {
        let virtual_path = virtual_path.to_str().unwrap().to_string();
        *self.virtual_path.write().unwrap() = virtual_path.clone();
        /*
        let parse_result = Parser::new(
            &shader_text,
            &virtual_path,
            Box::new(ShaderCIncluder::new()),
        );
        match parse_result {
            Ok(result) => *self.parsed_shader.write().unwrap() = Some(result),
            Err(error) => {
                *self.parsed_shader.write().unwrap() = None;
                logger::fatal!("shader syntax error : \n{}", error.to_string())
            }
        };
        self.permutations.write().unwrap().clear();

         */
    }

    pub fn get_program(&self, pass: &PassID) -> Option<Arc<dyn ShaderProgram>> {
        match self.permutations.read().unwrap().get(pass) {
            None => {}
            Some(permutation) => {
                return Some(permutation.shader.clone());
            }
        }

        /*
match &*self.parsed_shader.read().unwrap() {
    None => {}
    Some(parser) => {

        // Vertex shader
        let vertex_code = match parser.program_data.get_data(pass, &ShaderStage::Vertex) {
            Ok(code) => code,
            Err(error) => {
                logger::error!("failed to get vertex shader data :\n{}", error.to_string());
                return None;
            }
        };
        // Fragment shader
        let fragment_code = match parser.program_data.get_data(pass, &ShaderStage::Fragment)
        {
            Ok(code) => code,
            Err(error) => {
                logger::error!(
                    "failed to get fragment shader data :\n{}",
                    error.to_string()
                );
                return None;
            }
        };

        let vertex_sprv = match self.shader_backend.compile_to_spirv(
            vertex_code,
            Path::new(self.virtual_path.read().unwrap().as_str()),
            ShaderStage::Vertex,
            InterstageData {
                stage_outputs: Default::default(),
                binding_index: 0,
            },
        ) {
            Ok(sprv) => sprv,
            Err(error) => {
                logger::error!("Failed to compile vertex shader : \n{}", error.to_string());
                return None;
            }
        };

        let fragment_sprv = match self.shader_backend.compile_to_spirv(
            fragment_code,
            Path::new(self.virtual_path.read().unwrap().as_str()),
            ShaderStage::Fragment,
            InterstageData {
                stage_outputs: Default::default(),
                binding_index: 0,
            },
        ) {
            Ok(sprv) => sprv,
            Err(error) => {
                logger::error!(
                    "Failed to compile fragment shader : \n{}",
                    error.to_string()
                );
                return None;
            }
        };
        let _ci_shader = ShaderProgramInfos {
            vertex_stage: ShaderProgramStage {
                spirv: vertex_sprv.binary,
                descriptor_bindings: vertex_sprv.bindings,
                push_constant_size: vertex_sprv.push_constant_size,
                stage_input: vec![],
            },
            fragment_stage: ShaderProgramStage {
                spirv: fragment_sprv.binary,
                descriptor_bindings: fragment_sprv.bindings,
                push_constant_size: fragment_sprv.push_constant_size,
                stage_input: vec![],
            },
            shader_properties: parser.properties.clone(),
        };

        let render_pass = match crate::engine::Engine::get().gfx().find_render_pass(pass) {
            None => {
                logger::fatal!("trying to create shader program for render pass [{pass}], but this render pass is not available or registered")
            }
            Some(pass) => pass,
        };
        return None;

        let program = crate::engine::Engine::get().gfx().create_shader_program(
            self.meta_data.get_name(),
            &render_pass,
            &ci_shader,
        );
        self.permutations.write().unwrap().insert(
            pass.clone(),
            ShaderPermutation {
                shader: program.clone(),
            },
        );

        return Some(program);

            }
        }
         */
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
