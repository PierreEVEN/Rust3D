use std::sync::Arc;
use shader_base::{AlphaMode, Culling, FrontFace, PolygonMode, Topology};
use shader_base::spirv_reflector::DescriptorBinding;
use shader_base::types::{GfxCast, PixelFormat};
use crate::gfx::material::MaterialResourcePool;
use crate::gfx::shader_instance::ShaderInstance;

pub struct ShaderPropertyType {
    pub format: PixelFormat,
}

pub struct ShaderStageInput {
    pub location: i32,
    pub offset: u32,
    pub property_type: ShaderPropertyType,
}

pub struct ShaderProgramStage {
    pub spirv: Vec<u32>,
    pub descriptor_bindings: Vec<DescriptorBinding>,
    pub push_constant_size: u32,
    pub stage_input: Vec<ShaderStageInput>,
}

#[derive(Clone)]
pub struct ShaderProperties {
    pub shader_version: String,
    pub culling: Culling,
    pub front_face: FrontFace,
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
    pub alpha_mode: AlphaMode,
    pub depth_test: bool,
    pub line_width: f32,
}

impl Default for ShaderProperties {
    fn default() -> Self {
        Self {
            shader_version: "1.0".to_string(),
            culling: Default::default(),
            front_face: Default::default(),
            topology: Default::default(),
            polygon_mode: Default::default(),
            alpha_mode: Default::default(),
            depth_test: true,
            line_width: 1.0,
        }
    }
}

pub struct ShaderProgramInfos {
    pub vertex_stage: ShaderProgramStage,
    pub fragment_stage: ShaderProgramStage,
    pub shader_properties: ShaderProperties,
}

pub trait ShaderProgram: GfxCast {
    fn get_resources(&self) -> Arc<MaterialResourcePool>;
    fn instantiate(&self) -> Arc<dyn ShaderInstance>;
}

impl dyn ShaderProgram {
    pub fn cast<U: ShaderProgram + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}
