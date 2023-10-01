use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use display_json::DebugAsJsonPretty;
use serde::Serialize;
use crate::list_of::ListOf;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Instruction {
    Version(usize, u64),
    Pragma(usize, String, Value),
    Global(usize, RenderPassGroup, ListOf<HlslInstruction>),
    Vertex(usize, RenderPassGroup, ListOf<HlslInstruction>),
    Fragment(usize, RenderPassGroup, ListOf<HlslInstruction>),
    Compute(usize, RenderPassGroup, ListOf<HlslInstruction>),
}

#[derive(Debug)]
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

#[derive(Clone, Serialize, DebugAsJsonPretty)]
pub struct Register {
    pub content: String,
}

impl Register {
    pub fn new(content: &str) -> Self {
        Self { content: content.to_string() }
    }
}

#[derive(Debug)]
pub enum HlslInstruction {
    Struct(usize, String, Option<Register>, ListOf<StructureField>),
    Define(usize, String, Option<String>),
    Include(usize, String),
    Function(usize, String, Function),
    Property(usize, String, String, Option<Register>),
    Pragma(usize, String, Value),
}

#[derive(Debug)]
pub struct StructureField {
    pub struct_type: String,
    pub name: String,
    pub value: Option<String>,
    pub attribute: Option<String>,
}

#[derive(Debug)]
pub struct Function {
    pub return_type: String,
    pub attribute: Option<String>,
    pub params: ListOf<FunctionParameter>,
    pub content: ListOf<HlslCodeBlock>,
}

#[derive(Debug)]
pub struct FunctionParameter {
    pub param_type: String,
    pub name: String,
}

#[derive(Debug)]
pub enum HlslCodeBlock {
    InnerBlock(usize, ListOf<HlslCodeBlock>),
    Text(usize, String),
    Token(usize, char),
    Semicolon(usize),
}