use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use crate::GfxCast;
use crate::shader_instance::BindPoint;

use crate::types::PixelFormat;

#[derive(Clone)]
pub struct PassID {
    #[cfg(not(debug_assertions))]
    internal_id: u64,
    
    #[cfg(debug_assertions)]
    internal_id: String,
}

impl PassID {
    pub fn new(id_name: &str) -> PassID {
        #[cfg(not(debug_assertions))]
        {
            let mut hasher = DefaultHasher::new();
            id_name.hash(&mut hasher);
            Self {
                internal_id: hasher.finish(),
            }
        }
        #[cfg(debug_assertions)]
        Self {
            internal_id: id_name.to_string()
        }
    }
}

impl Hash for PassID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[cfg(not(debug_assertions))]
        state.write_u64(self.internal_id);
        
        #[cfg(debug_assertions)]
        state.write(self.internal_id.as_bytes());
    }
}

#[cfg(debug_assertions)]
impl PartialEq<Self> for PassID {
    fn eq(&self, other: &Self) -> bool {
        self.internal_id == other.internal_id
    }
}

#[cfg(not(debug_assertions))]
impl PartialEq<Self> for PassID {
    fn eq(&self, other: &Self) -> bool {
        self.internal_id == other.internal_id
    }
}

impl Display for PassID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.internal_id);
    }
}

impl Eq for PassID {}

pub struct ShaderPropertyType {
    pub format: PixelFormat,
}

pub struct ShaderStageInput {
    pub location: i32,
    pub offset: u32,
    pub property_type: ShaderPropertyType,
}

#[derive(Clone)]
pub struct DescriptorBinding {
    pub bind_point: BindPoint,
    pub binding: u32,
    pub descriptor_type: DescriptorType,
}

pub struct ShaderProgramStage {
    pub spirv: Vec<u32>,
    pub descriptor_bindings: Vec<DescriptorBinding>,
    pub push_constant_size: u32,
    pub stage_input: Vec<ShaderStageInput>,
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

pub trait ShaderProgram : GfxCast {
    fn get_bindings(&self) -> Vec<DescriptorBinding>;
    
}