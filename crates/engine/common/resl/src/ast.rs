use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use regex::Regex;
use shader_base::ShaderStage;
use crate::list_of::ListOf;

#[derive(Debug, Clone)]
pub struct RenderPassGroup {
    render_passes: HashSet<String>,
}

impl RenderPassGroup {
    pub fn new() -> Self { Self { render_passes: Default::default() } }
    pub fn add(mut self, pass: String) -> Self {
        self.render_passes.insert(pass);
        self
    }
    pub fn iter(&self) -> impl Iterator<Item=&String> {
        return self.render_passes.iter();
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Version(usize, u64),
    Pragma(usize, String, Value),
    Global(usize, RenderPassGroup, ListOf<HlslInstruction>),
    Block(usize, ShaderStage, RenderPassGroup, ListOf<HlslInstruction>),
}

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Integer(i64),
    Float(String),
    String(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => { f.write_str("") }
            Value::Integer(i) => { f.write_str(i.to_string().as_str()) }
            Value::Float(d) => { f.write_str(d.to_string().as_str()) }
            Value::String(s) => { f.write_str(s.to_string().as_str()) }
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum HlslInstruction {
    Struct(usize, String, ListOf<StructureField>),
    Define(usize, String, Option<String>),
    Include(usize, String),
    Function(usize, String, Function),
    Property(usize, HlslType, String),
    Pragma(usize, String, Value),
}

#[derive(Debug, Clone)]
pub struct StructureField {
    pub token: usize,
    pub struct_type: HlslType,
    pub name: String,
    pub value: Option<String>,
    pub attribute: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub return_type: (usize, HlslType), // token, attribute
    pub attribute: Option<(usize, String)>, // token, attribute
    pub params: ListOf<FunctionParameter>,
    pub content: (usize, usize, ListOf<HlslCodeBlock>), // begin, end, content
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub param_type: (usize, HlslType), // token, type
    pub name: (usize, String),  // token, name
    pub attribute: Option<(usize, String)>
}

#[derive(Debug, Clone)]
pub enum HlslCodeBlock {
    InnerBlock(usize, usize, ListOf<HlslCodeBlock>),
    Text(usize, String),
    Token(usize, char),
    Semicolon(usize),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HlslTypeSimple {
    Void,
    Bool,
    Int,
    Uint,
    Byte,
    Half,
    Float,
    Double,
    Texture(u8),
    Struct(String),
    Buffer,
    ConstantBuffer,
    PushConstant,
    SamplerState,
    ResourceImage,
    SamplerComparisonState,
}

impl Display for HlslTypeSimple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HlslTypeSimple::Void => { f.write_str("void") }
            HlslTypeSimple::Bool => { f.write_str("bool") }
            HlslTypeSimple::Int => { f.write_str("int") }
            HlslTypeSimple::Uint => { f.write_str("uint") }
            HlslTypeSimple::Byte => { f.write_str("float") }
            HlslTypeSimple::Half => { f.write_str("half") }
            HlslTypeSimple::Float => { f.write_str("float") }
            HlslTypeSimple::Double => { f.write_str("double") }
            HlslTypeSimple::Struct(str) => { f.write_str(str.as_str()) }
            HlslTypeSimple::Texture(c) => { f.write_fmt(format_args!("Texture{c}D")) }
            HlslTypeSimple::Buffer => { f.write_str("Buffer") }
            HlslTypeSimple::ConstantBuffer => { f.write_str("ConstantBuffer") }
            HlslTypeSimple::PushConstant => { f.write_str("PushConstant") }
            HlslTypeSimple::SamplerState => { f.write_str("SamplerState") }
            HlslTypeSimple::SamplerComparisonState => { f.write_str("SamplerComparisonState") }
            HlslTypeSimple::ResourceImage => { f.write_str("Texture2D") }
        }
    }
}

#[derive(Debug, Clone)]
pub enum HlslType {
    Simple(HlslTypeSimple),
    Vec(HlslTypeSimple, u8),
    Mat(HlslTypeSimple, u8, u8),
    Template(HlslTypeSimple, Vec<HlslType>),
}

impl Display for HlslType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HlslType::Simple(s) => { s.fmt(f) }
            HlslType::Vec(t, u) => {f.write_fmt(format_args!("{}{u}", t))}
            HlslType::Mat(t, u, v) => {f.write_fmt(format_args!("{}{u}x{v}", t))}
            HlslType::Template(t, inner) => {
                let mut types = String::new();
                for inner in inner.iter() {
                    types += format!("{},", inner).as_str()
                }
                f.write_fmt(format_args!("{}<{}>", t, types))
            }
        }
    }
}

impl HlslType {
    fn num_at(hlsl: &str, offset: usize) -> u8 {
        hlsl.chars().nth(offset).unwrap().to_string().parse().unwrap()
    }

    fn vec(base_type: HlslTypeSimple, hlsl: &str) -> HlslType {
        if let HlslTypeSimple::Byte = base_type {
            return HlslType::Vec(base_type, Self::num_at(hlsl, 4))
        }
        HlslType::Vec(base_type.clone(), Self::num_at(hlsl, base_type.to_string().len()))
    }
    fn mat(base_type: HlslTypeSimple, hlsl: &str) -> HlslType {
        let base_length = base_type.to_string().len();
        HlslType::Mat(base_type, Self::num_at(hlsl, base_length), Self::num_at(hlsl, base_length + 2))
    }

    fn template_type(hlsl: &str) -> HlslType {
        let mut base = hlsl.split('<');
        let name = base.next().unwrap();
        let name_type = match Self::from(name) {
            HlslType::Simple(s) => { s }
            _ => panic!("Unrecognized template type format : {hlsl}")
        };
        let types = base.next().unwrap().split('>').next().unwrap();
        if types.contains(',') {
            let mut args = vec![];
            for arg in types.split(',') {
                args.push(Self::from(arg));
            }
            HlslType::Template(name_type, args)
        } else {
            HlslType::Template(name_type, vec![Self::from(types)])
        }
    }
}

impl From<&str> for HlslType {
    fn from(value: &str) -> Self {
        match value {
            "void" => HlslType::Simple(HlslTypeSimple::Void),
            "bool" => HlslType::Simple(HlslTypeSimple::Bool),
            "int" => HlslType::Simple(HlslTypeSimple::Uint),
            "uint" => HlslType::Simple(HlslTypeSimple::Int),
            "byte" => HlslType::Simple(HlslTypeSimple::Byte),
            "half" => HlslType::Simple(HlslTypeSimple::Half),
            "float" => HlslType::Simple(HlslTypeSimple::Float),
            "double" => HlslType::Simple(HlslTypeSimple::Double),
            "Texture" => HlslType::Simple(HlslTypeSimple::Texture(0)),
            "Texture1D" => HlslType::Simple(HlslTypeSimple::Texture(1)),
            "Texture2D" => HlslType::Simple(HlslTypeSimple::Texture(2)),
            "Texture3D" => HlslType::Simple(HlslTypeSimple::Texture(3)),
            "PushConstant" => HlslType::Simple(HlslTypeSimple::PushConstant),
            "Buffer" => HlslType::Simple(HlslTypeSimple::Buffer),
            "ConstantBuffer" => HlslType::Simple(HlslTypeSimple::ConstantBuffer),
            "SamplerState" => HlslType::Simple(HlslTypeSimple::SamplerState),
            "ResourceImage" => HlslType::Simple(HlslTypeSimple::ResourceImage),
            "SamplerComparisonState" => HlslType::Simple(HlslTypeSimple::SamplerComparisonState),
            _ => {
                if Regex::new("^bool[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Bool, value)
                } else if Regex::new("^int[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Int, value)
                } else if Regex::new("^uint[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Uint, value)
                } else if Regex::new("^byte[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Byte, value)
                } else if Regex::new("^half[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Half, value)
                } else if Regex::new("^float[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Float, value)
                } else if Regex::new("^double[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Double, value)
                } else if Regex::new("^bool[1-4]x[1-4]$").unwrap().is_match(value) {
                    Self::mat(HlslTypeSimple::Bool, value)
                } else if Regex::new("^int[1-4]x[1-4]$").unwrap().is_match(value) {
                    Self::mat(HlslTypeSimple::Int, value)
                } else if Regex::new("^uint[1-4]x[1-4]$").unwrap().is_match(value) {
                    Self::mat(HlslTypeSimple::Uint, value)
                } else if Regex::new("^byte[1-4]x[1-4]$").unwrap().is_match(value) {
                    Self::mat(HlslTypeSimple::Float, value)
                } else if Regex::new("^half[1-4]x[1-4]$").unwrap().is_match(value) {
                    Self::mat(HlslTypeSimple::Half, value)
                } else if Regex::new("^float[1-4]x[1-4]$").unwrap().is_match(value) {
                    Self::mat(HlslTypeSimple::Float, value)
                } else if Regex::new("^matrix[1-4]x[1-4]$").unwrap().is_match(value) {
                    Self::mat(HlslTypeSimple::Float, &value[1..])
                } else if Regex::new("^double[1-4]x[1-4]$").unwrap().is_match(value) {
                    Self::mat(HlslTypeSimple::Double, value)
                } else if Regex::new("^[a-zA-Z][a-zA-Z0-9_]*<([a-zA-Z][a-zA-Z0-9_]|,)*>$").unwrap().is_match(value) {
                    Self::template_type(value)
                } else if Regex::new("^[a-zA-Z][a-zA-Z0-9_]*$").unwrap().is_match(value) {
                    Self::Simple(HlslTypeSimple::Struct(value.to_string()))
                } else {
                    panic!("Unrecognized Hlsl type")
                }
            }
        }
    }
}