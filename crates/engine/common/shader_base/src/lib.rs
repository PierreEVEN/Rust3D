use crate::pass_id::PassID;

pub mod pass_id;

pub trait ShaderInterface {
    fn get_spirv_for(&self, render_pass: &PassID, stage: ShaderStage) -> Vec<u32>;

    fn get_parameters_for(&self, render_pass: &PassID) -> &ShaderParameters;
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

#[derive(Clone, Debug, Default)]
pub struct ShaderParameters {
    pub alpha_mode: AlphaMode,
    pub polygon_mode: PolygonMode,
    pub topology: Topology,
    pub front_face: FrontFace,
    pub culling: Culling
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Debug)]
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