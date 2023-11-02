use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::pass_id::PassID;
use crate::types::PixelFormat;

pub mod pass_id;
pub mod spirv_reflector;
pub mod types;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct BindPoint {
    pub name: String,
}

impl BindPoint {
    pub fn new(name: &str) -> BindPoint {
        BindPoint {
            name: name.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ShaderResource {
    pub resource_type: DescriptorType,
    pub locations: HashMap<PassID, HashMap<ShaderStage, u32>>, // location per stage per pass
}

#[derive(Default, Clone)]
pub struct ShaderResourcePool {
    resources: HashMap<BindPoint, ShaderResource>,
}

impl ShaderResourcePool {
    pub fn add_resource(&mut self, bp: &BindPoint, resource_type: DescriptorType, stage: &ShaderStage, pass: &PassID, location: u32) -> Result<(), CompilationError> {
        let binding_type = self.resources.entry(bp.clone()).or_insert(ShaderResource { resource_type: resource_type.clone(), locations: Default::default() });
        if binding_type.resource_type != resource_type {
            return Err(CompilationError::throw(format!("Found multiple resources with the same name but different types : {:?} - {:?}", binding_type.resource_type, resource_type), None));
        }

        binding_type.locations.entry(pass.clone()).or_insert(HashMap::default())
            .entry(stage.clone()).or_insert(location);

        Ok(())
    }
    pub fn get_resources(&self) -> &HashMap<BindPoint, ShaderResource> {
        &self.resources
    }
    pub fn get_binding_for_pass(&self, pass_id: &PassID) -> Vec<(ShaderStage, DescriptorType, u32)>{
        let mut bindings = vec![];
        for resource in self.resources.values() {
            match resource.locations.get(pass_id) {
                None => {}
                Some(binding) => {
                    for (stage, location) in binding {
                        bindings.push((stage.clone(), resource.resource_type.clone(), *location));
                    }
                }
            }
        }
        bindings
    }
}

pub trait ShaderInterface {
    // Generate and retrieve spirv code for the given pass and stage if available
    fn get_spirv_for(&self, render_pass: &PassID, stage: &ShaderStage) -> Result<Vec<u32>, CompilationError>;
    // Get shader parameter for given pass
    fn get_parameters_for(&self, render_pass: &PassID) -> &ShaderParameters;
    // Get stage input properties
    fn get_stage_inputs(&self, render_pass: &PassID, stage: &ShaderStage) -> Result<Vec<Property>, String>;
    // Get stage output properties
    fn get_stage_outputs(&self, render_pass: &PassID, stage: &ShaderStage) -> Result<Vec<Property>, String>;
    // Get parse and compilation errors
    fn get_errors(&self) -> &Vec<CompilationError>;
    // Each different shader interface should have a different path
    fn get_path(&self) -> PathBuf;
    // Get shader bindings
    fn resource_pool(&self) -> &Arc<ShaderResourcePool>;
    fn get_entry_point(&self, render_pass: &PassID, stage: &ShaderStage) -> Result<String, String>;
    fn push_constant_size(&self, stage: &ShaderStage, pass: &PassID) -> Option<u32>;
}

pub struct Property {
    pub name: String,
    pub format: PixelFormat,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    TesselationControl,
    TesselationEvaluate,
    Geometry,
    Compute,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum Culling {
    None,
    Front,
    #[default]
    Back,
    Both,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum FrontFace {
    Clockwise,
    #[default]
    CounterClockwise,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum Topology {
    Points,
    Lines,
    #[default]
    Triangles,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum PolygonMode {
    Point,
    Line,
    #[default]
    Fill,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Translucent,
    Additive,
}

#[derive(Clone, Debug)]
pub struct ShaderParameters {
    pub alpha_mode: AlphaMode,
    pub polygon_mode: PolygonMode,
    pub topology: Topology,
    pub front_face: FrontFace,
    pub culling: Culling,
    pub line_width: f32,
    pub depth_test: bool,
}

impl Default for ShaderParameters {
    fn default() -> Self {
        Self {
            alpha_mode: Default::default(),
            polygon_mode: Default::default(),
            topology: Default::default(),
            front_face: Default::default(),
            culling: Default::default(),
            line_width: 1.0,
            depth_test: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DescriptorType {
    Sampler,
    CombinedImageSampler,
    SampledImage,
    StorageImage,
    UniformTexelBuffer,
    StorageTexelBuffer,
    UniformBuffer,
    StorageBuffer,
    UniformBufferDynamic,
    StorageBufferDynamic,
    InputAttachment,
}

#[derive(Debug, Clone)]
pub struct CompilationError {
    pub message: String,
    pub token: Option<usize>,
}

impl CompilationError {
    pub fn throw(message: String, token: Option<usize>) -> Self {
        Self {
            message,
            token,
        }
    }
}
