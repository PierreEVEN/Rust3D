use std::fmt;
use std::fmt::{Display, Formatter};

use crate::types::PixelFormat;

pub type PassID = String;

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
    pub push_constant_size: u32,
    pub stage_input: Vec<ShaderStageInput>,
}

pub struct ShaderProgramInfos {
    pub vertex_stage: ShaderProgramStage,
    pub fragment_stage: ShaderProgramStage,

    pub culling: Culling,
    pub front_face: FrontFace,
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
    pub alpha_mode: AlphaMode,
    pub depth_test: bool,
    pub line_width: f32,
}

pub struct ShaderBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Culling
{
    None,
    Front,
    Back,
    Both,
}

impl Default for Culling {
    fn default() -> Self {
        Culling::Back
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrontFace
{
    Clockwise,
    CounterClockwise,
}

impl Default for FrontFace {
    fn default() -> Self {
        FrontFace::CounterClockwise
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Topology
{
    Points,
    Lines,
    Triangles,
}

impl Default for Topology {
    fn default() -> Self {
        Topology::Triangles
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolygonMode
{
    Point,
    Line,
    Fill,
}

impl Default for PolygonMode {
    fn default() -> Self {
        PolygonMode::Fill
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlphaMode
{
    Opaque,
    Translucent,
    Additive,
}

impl Default for AlphaMode {
    fn default() -> Self {
        AlphaMode::Opaque
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ShaderStage
{
    Vertex,
    Fragment,
}

impl Display for ShaderStage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ShaderStage::Vertex => write!(f, "Vertex"),
            ShaderStage::Fragment => write!(f, "Fragment"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DescriptorType
{
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