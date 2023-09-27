use std::collections::HashSet;

#[derive(Debug)]
pub struct ListOf<T> {
    items: Vec<T>,
}

impl<T> ListOf<T> {
    pub fn new() -> Self { Self { items: vec![] } }

    pub fn push(mut self, instr: T) -> Self {
        self.items.push(instr);
        self
    }

    pub fn concat(mut self, other: &mut Self) -> Self {
        self.items.append(&mut other.items);
        self
    }
}

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
}

#[derive(Debug)]
pub enum Instruction {
    Pragma(String, Value),
    Global(RenderPassGroup, ListOf<HlslInstruction>),
    Vertex(RenderPassGroup, ListOf<HlslInstruction>),
    Fragment(RenderPassGroup, ListOf<HlslInstruction>),
    Compute(RenderPassGroup, ListOf<HlslInstruction>),
}

#[derive(Debug)]
pub enum Value {
    None,
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug)]
pub enum HlslInstruction {
    Struct(String, ListOf<StructureField>),
    Function(String, Function),
    Pragma(String, Value)
}

#[derive(Debug)]
pub struct StructureField {
    pub struct_type: String,
    pub name: String,
    pub value: Option<String>,
    pub attribute: Option<String>
}

#[derive(Debug)]
pub struct Function {
    pub return_type: String,
    pub params: Vec<FunctionParameter>,
    pub content: String,
}

#[derive(Debug)]
pub struct FunctionParameter {
    pub param_type: String,
    pub name: String,
}
