use crate::pass_id::PassID;

pub mod pass_id;

pub trait ShaderInterface {
    fn get_spirv_for(&self, _render_pass: &PassID, _stage: ShaderStage) -> Vec<u8>;
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
pub enum ShaderLanguage {
    #[default]
    HLSL,
    GLSL,
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