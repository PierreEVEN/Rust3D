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

#[derive(Clone, Debug)]
pub struct Register {
    pub content: String,
}

impl Register {
    pub fn new(content: &str) -> Self {
        Self { content: content.to_string() }
    }
}

#[derive(Debug, Clone)]
pub enum HlslInstruction {
    Struct(usize, String, Option<Register>, ListOf<StructureField>),
    Define(usize, String, Option<String>),
    Include(usize, String),
    Function(usize, String, Function),
    Property(usize, HlslType, String, Option<Register>),
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

#[derive(Debug, Clone)]
pub enum HlslTypeSimple {
    Void,
    Bool,
    Int,
    Uint,
    Half,
    Float,
    Double,
    Texture(u8),
    Struct(String),
    Buffer,
    ConstantBuffer,
    PushConstant,
    SamplerState,
    SamplerComparisonState,
}

impl HlslTypeSimple {
    pub fn to_string(&self) -> String {
        match self {
            HlslTypeSimple::Void => { "void".to_string() }
            HlslTypeSimple::Bool => { "bool".to_string() }
            HlslTypeSimple::Int => { "int".to_string() }
            HlslTypeSimple::Uint => { "uint".to_string() }
            HlslTypeSimple::Half => { "half".to_string() }
            HlslTypeSimple::Float => { "float".to_string() }
            HlslTypeSimple::Double => { "double".to_string() }
            HlslTypeSimple::Struct(str) => { str.clone() }
            HlslTypeSimple::Texture(c) => { format!("Texture{c}D") }
            HlslTypeSimple::Buffer => { "Buffer".to_string() }
            HlslTypeSimple::ConstantBuffer => { "ConstantBuffer".to_string() }
            HlslTypeSimple::PushConstant => { "PushConstant".to_string() }
            HlslTypeSimple::SamplerState => { "SamplerState".to_string() }
            HlslTypeSimple::SamplerComparisonState => { "SamplerComparisonState".to_string() }
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

impl HlslType {
    pub fn to_string(&self) -> String {
        match self {
            HlslType::Simple(s) => {s.to_string()}
            HlslType::Vec(t, u) => {format!("{}{u}", t.to_string())}
            HlslType::Mat(t, u, v) => {format!("{}{u}x{v}", t.to_string())}
            HlslType::Template(t, inner) => {
                let mut types = String::new();
                for inner in inner.iter() {
                    types += format!("{},", inner.to_string()).as_str()
                }
                format!("{}<{}>", t.to_string(), types)
            }
        }
    }
}
impl HlslType {
    fn num_at(hlsl: &str, offset: usize) -> u8 {
        hlsl.chars().nth(offset).unwrap().to_string().parse().unwrap()
    }

    fn vec(base_type: HlslTypeSimple, hlsl: &str) -> HlslType {
        HlslType::Vec(base_type.clone(), Self::num_at(hlsl, base_type.to_string().len()))
    }
    fn mat(base_type: HlslTypeSimple, hlsl: &str) -> HlslType {
        let base_length = base_type.to_string().len();
        HlslType::Mat(base_type, Self::num_at(hlsl, base_length), Self::num_at(hlsl, base_length + 2))
    }

    fn template_type(hlsl: &str) -> HlslType {
        let mut base = hlsl.split("<");
        let name = base.next().unwrap();
        let name_type = match Self::from(name) {
            HlslType::Simple(s) => { s }
            _ => panic!("Unrecognized template type format : {hlsl}")
        };
        let types = base.next().unwrap().split(">").nth(0).unwrap();
        if types.contains(",") {
            let mut args = vec![];
            for arg in types.split(",") {
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
        let res = match value {
            "void" => HlslType::Simple(HlslTypeSimple::Void),
            "bool" => HlslType::Simple(HlslTypeSimple::Bool),
            "int" => HlslType::Simple(HlslTypeSimple::Uint),
            "uint" => HlslType::Simple(HlslTypeSimple::Int),
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
            "SamplerComparisonState" => HlslType::Simple(HlslTypeSimple::SamplerComparisonState),
            _ => {
                if Regex::new("^bool[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Bool, value)
                } else if Regex::new("^int[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Int, value)
                } else if Regex::new("^uint[1-4]$").unwrap().is_match(value) {
                    Self::vec(HlslTypeSimple::Uint, value)
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
        };
        res
    }
}