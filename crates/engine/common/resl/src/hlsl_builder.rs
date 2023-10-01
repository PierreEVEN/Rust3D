use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use display_json::DebugAsJsonPretty;
use lalrpop_util::lalrpop_mod;
use serde::Serialize;

use crate::ast::{Function, HlslCodeBlock, HlslInstruction, Instruction, Register, RenderPassGroup, StructureField};
use crate::list_of::ListOf;

lalrpop_mod!(pub language);

#[derive(Serialize, DebugAsJsonPretty)]
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

#[derive(Default, Serialize, DebugAsJsonPretty)]
pub struct ReslParser {
    version: Option<u64>,
    pragmas: HashMap<String, String>,
    file_path: PathBuf,
    resl: String,
    pub hlsl: HashMap<String, HashMap<String, ReslBlock>>,
}

#[derive(Clone, Serialize, DebugAsJsonPretty)]
pub struct Token {
    pub location: usize,
    pub value: String,
}

impl From<(usize, &String)> for Token {
    fn from(value: (usize, &String)) -> Self {
        let (location, value) = value;
        Self { location, value: value.clone() }
    }
}

impl From<(usize, &str)> for Token {
    fn from(value: (usize, &str)) -> Self {
        let (location, value) = value;
        Self { location, value: value.to_string() }
    }
}

#[derive(Default, Serialize, DebugAsJsonPretty)]
pub struct ReslBlock {
    tokens: Vec<Token>,
    defines: HashMap<String, Option<String>>,
    pragmas: HashMap<String, String>,
    includes: HashSet<String>,
    warnings: Vec<CompilationError>,
    functions: HashMap<String, (String, Vec<String>)>,
    structs: HashMap<String, (Option<Register>, Vec<(String, String)>)>,
    properties: HashMap<String, String>,
}

impl ReslBlock {
    pub fn append(&mut self, other: &Self) {
        for token in &other.tokens {
            self.tokens.push(token.clone())
        }
    }
}

impl ReslBlock {
    fn tokenize_hlsl(&mut self, content: &ListOf<HlslCodeBlock>) {
        for block in content.iter() {
            match block {
                HlslCodeBlock::InnerBlock(t, inner) => {
                    self.tokens.push(Token::from((*t, "{")));
                    self.tokenize_hlsl(inner);
                    self.tokens.push(Token::from((*t, "}")));
                }
                HlslCodeBlock::Text(t, text) => { self.tokens.push(Token::from((*t, text))); }
                HlslCodeBlock::Token(t, token) => { self.tokens.push(Token::from((*t, &token.to_string()))); }
                HlslCodeBlock::Semicolon(t) => { self.tokens.push(Token::from((*t, ";"))); }
            }
        }
    }

    fn push_function(&mut self, token: usize, name: &String, content: &Function) -> Result<(), CompilationError> {
        match self.functions.get(name) {
            None => {
                self.tokens.push(Token::from((token, &content.return_type)));
                self.tokens.push(Token::from((token, name)));
                match &content.attribute {
                    None => {}
                    Some(attribute) => {
                        self.tokens.push(Token::from((token, ":")));
                        self.tokens.push(Token::from((token, attribute)));                        
                    }
                }
                self.tokens.push(Token::from((token, "(")));
                let mut params = vec![];
                for param in content.params.iter() {
                    self.tokens.push(Token::from((token, &param.param_type)));
                    self.tokens.push(Token::from((token, &param.name)));
                    params.push(param.param_type.clone());
                }
                self.tokens.push(Token::from((token, ")")));
                self.tokens.push(Token::from((token, "{")));
                self.tokenize_hlsl(&content.content);
                self.tokens.push(Token::from((token, "}")));
                self.functions.insert(name.clone(), (content.return_type.clone(), params));
            }
            Some(_) => {
                return Err(CompilationError {
                    message: format!("Function redeclaration : {}", name),
                    token: Some(token),
                });
            }
        }
        Ok(())
    }

    fn push_struct(&mut self, token: usize, name: &String, register: &Option<Register>, content: &ListOf<StructureField>) -> Result<(), CompilationError> {
        match self.functions.get(name) {
            None => {
                self.tokens.push(Token::from((token, "struct")));
                self.tokens.push(Token::from((token, name)));
                match register {
                    None => {}
                    Some(register) => {
                        self.tokens.push(Token::from((token, ":")));
                        self.tokens.push(Token::from((token, &register.content)));
                    }
                }
                self.tokens.push(Token::from((token, "{")));
                let mut params = vec![];
                for param in content.iter() {
                    self.tokens.push(Token::from((token, &param.struct_type)));
                    self.tokens.push(Token::from((token, &param.name)));
                    match &param.value {
                        None => {}
                        Some(value) => {
                            self.tokens.push(Token::from((token, "=")));
                            self.tokens.push(Token::from((token, value)));
                        }
                    }
                    match &param.attribute {
                        None => {}
                        Some(attribute) => {
                            self.tokens.push(Token::from((token, ":")));
                            self.tokens.push(Token::from((token, attribute)));
                        }
                    }
                    self.tokens.push(Token::from((token, ";")));
                    params.push((param.struct_type.clone(), param.name.clone()));
                }
                self.tokens.push(Token::from((token, "}")));
                self.tokens.push(Token::from((token, ";")));
                self.structs.insert(name.clone(), (register.clone(), params));
            }
            Some(_) => {
                return Err(CompilationError {
                    message: format!("Structure redeclaration : {}", name),
                    token: Some(token),
                });
            }
        }
        Ok(())
    }

    fn push_property(&mut self, token: usize, prop_type: &String, name: &String, register: &Option<Register>) -> Result<(), CompilationError> {
        match self.functions.get(name) {
            None => {
                self.tokens.push(Token::from((token, prop_type)));
                self.tokens.push(Token::from((token, name)));
                match register {
                    None => {}
                    Some(register) => {
                        self.tokens.push(Token::from((token, ":")));
                        self.tokens.push(Token::from((token, &register.content)));
                    }
                }
                self.tokens.push(Token::from((token, ";")));
                self.properties.insert(name.clone(), prop_type.clone());
            }
            Some(_) => {
                return Err(CompilationError {
                    message: format!("Property redeclaration : {}", name),
                    token: Some(token),
                });
            }
        }
        Ok(())
    }

    pub fn parse(&mut self, content: &ListOf<HlslInstruction>) -> Result<(), CompilationError> {
        for instruction in content.iter() {
            match instruction {
                HlslInstruction::Struct(t, name, register, fields) => {
                    match self.push_struct(*t, name, register, fields) {
                        Ok(_) => {}
                        Err(err) => { return Err(err); }
                    }
                }
                HlslInstruction::Define(token, key, value) => {
                    self.tokens.push(Token::from((*token, "#define")));
                    self.tokens.push(Token::from((*token, key)));
                    match value {
                        None => {
                            self.tokens.push(Token::from((*token, "\n")));
                            self.defines.insert(key.clone(), value.clone());
                        }
                        Some(value) => {
                            self.tokens.push(Token::from((*token, &format!("{}\n", value))));
                            self.defines.insert(key.clone(), Some(value.clone()));
                        }
                    }
                }
                HlslInstruction::Include(t, file) => {
                    self.tokens.push(Token::from((*t, "#include")));
                    self.tokens.push(Token::from((*t, &format!("\"{}\"\n", file))));
                    if self.includes.contains(file) { self.warnings.push(CompilationError::throw(format!("Include duplication : {}", file), Some(*t))) }
                    self.includes.insert(file.clone());
                }
                HlslInstruction::Function(t, name, content) => {
                    match self.push_function(*t, name, content) {
                        Ok(_) => {}
                        Err(err) => { return Err(err); }
                    }
                }
                HlslInstruction::Property(t, prop_type, name, register) => {
                    match self.push_property(*t, prop_type, name, register) {
                        Ok(_) => {}
                        Err(err) => { return Err(err); }
                    }
                }
                HlslInstruction::Pragma(t, key, value) => {
                    let value = format!("{}", value);
                    self.tokens.push(Token::from((*t, "#include")));
                    self.tokens.push(Token::from((*t, key)));
                    self.tokens.push(Token::from((*t, &value)));
                    match self.pragmas.get(key) {
                        None => { self.pragmas.insert(key.clone(), value); }
                        Some(existing) => {
                            if *existing != value {
                                return Err(CompilationError {
                                    message: format!("Pragma redeclaration {} (old : {})", value, existing),
                                    token: Some(*t),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl ReslParser {
    pub fn parse(&mut self, resl_code: String, file_path: PathBuf) -> Result<ListOf<Instruction>, CompilationError> {
        self.resl = resl_code;
        self.file_path = file_path;
        let code = match language::InstructionListParser::new().parse(self.resl.as_str()) {
            Ok(code) => code,
            Err(e) => return match e {
                lalrpop_util::ParseError::UnrecognizedToken { token, expected } => {
                    let (start, found, _) = token;
                    Err(CompilationError {
                        message: format!("Unrecognized token {:?} : expected {:?}", found, expected),
                        token: Some(start),
                    })
                }
                lalrpop_util::ParseError::InvalidToken { location } => {
                    Err(CompilationError {
                        message: format!("Invalid token {:?}", self.resl.chars().nth(location).unwrap()),
                        token: Some(location),
                    })
                }
                _ => {
                    Err(CompilationError {
                        message: format!("{}", e),
                        token: None,
                    })
                }
            }
        };
        match self.internal_analyze(&code) {
            Ok(_) => {}
            Err(err) => { return Err(err); }
        };
        Ok(code)
    }

    fn internal_analyze_block(&mut self, block_type: &str, render_pass_groups: &RenderPassGroup, content: &ListOf<HlslInstruction>) -> Result<(), CompilationError> {
        for pass in render_pass_groups.iter() {
            match self.hlsl.get_mut(pass) {
                None => {
                    if block_type != "global" {
                        let mut block = ReslBlock::default();
                        match block.parse(content) {
                            Ok(_) => {}
                            Err(err) => { return Err(err) }
                        }
                        self.hlsl.insert(pass.clone(), HashMap::from([(block_type.to_string(), block)]));
                    }
                }
                Some(pass) => {
                    if block_type == "global" {
                        for pass in pass.values_mut() {
                            match pass.parse(content) {
                                Ok(_) => {}
                                Err(err) => { return Err(err) }
                            }
                        }
                    }
                    else {
                        match pass.get_mut(&block_type.to_string()) {
                            None => {
                                let mut block = ReslBlock::default();
                                match block.parse(content) {
                                    Ok(_) => {}
                                    Err(err) => { return Err(err) }
                                }
                                pass.insert(block_type.to_string(), block);
                            }
                            Some(stage) => {
                                match stage.parse(content) {
                                    Ok(_) => {}
                                    Err(err) => { return Err(err) }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn pre_alocate_stages(&mut self, block_type: &str, rpg: &RenderPassGroup) {
        for pass in rpg.iter() {
            match self.hlsl.get_mut(pass) {
                None => { self.hlsl.insert(pass.clone(), HashMap::from([(block_type.to_string(), ReslBlock::default())])); }
                Some(pass) => {
                    match pass.get_mut(&block_type.to_string()) {
                        None => { pass.insert(block_type.to_string(), ReslBlock::default()); }
                        Some(_) => {}
                    }
                }
            }
        }
    }

    fn internal_analyze(&mut self, instructions: &ListOf<Instruction>) -> Result<(), CompilationError> {

        // Allocate blocks
        for instruction in instructions.iter() {
            match instruction {
                Instruction::Version(_, _) => {}
                Instruction::Pragma(_, _, _) => {}
                Instruction::Global(_, _, _) => {}
                Instruction::Vertex(_, rpg, _) => { self.pre_alocate_stages("vertex", rpg); }
                Instruction::Fragment(_, rpg, _) => { self.pre_alocate_stages("fragment", rpg); }
                Instruction::Compute(_, rpg, _) => { self.pre_alocate_stages("compute", rpg); }
            }
        }

        for instruction in instructions.iter() {
            match instruction {
                Instruction::Version(t, version) => {
                    match self.version {
                        None => { self.version = Some(*version) }
                        Some(current_version) => {
                            if current_version != *version {
                                return Err(CompilationError::throw(
                                    format!("Version redeclaration with different value : {} (old : {})", version, current_version), Some(*t)));
                            }
                        }
                    }
                }
                Instruction::Pragma(t, key, value) => {
                    let new_value = format!("{}", value);
                    match self.pragmas.get(key) {
                        None => { self.pragmas.insert(key.clone(), new_value); }
                        Some(existing) => {
                            if *existing != new_value {
                                return Err(CompilationError::throw(
                                    format!("Pragma version with different value : {} = {} (old : {})", key, new_value, existing), Some(*t)));
                            }
                        }
                    }
                }
                Instruction::Global(_, rpg, content) => {
                    match self.internal_analyze_block("global", rpg, content) {
                        Ok(_) => {}
                        Err(err) => { return Err(err); }
                    }
                }
                Instruction::Vertex(_, rpg, content) => {
                    match self.internal_analyze_block("vertex", rpg, content) {
                        Ok(_) => {}
                        Err(err) => { return Err(err); }
                    }
                }
                Instruction::Fragment(_, rpg, content) => {
                    match self.internal_analyze_block("fragment", rpg, content) {
                        Ok(_) => {}
                        Err(err) => { return Err(err); }
                    }
                }
                Instruction::Compute(_, rpg, content) => {
                    match self.internal_analyze_block("compute", rpg, content) {
                        Ok(_) => {}
                        Err(err) => { return Err(err); }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_error_location(&self, token: usize) -> (usize, usize) {
        let mut line = 1;
        let mut token_to_last_line = 0;
        let mut elapsed_tokens = 0;
        for chr in self.resl.chars() {
            if chr == '\n' {
                line += 1;
                token_to_last_line = elapsed_tokens;
            }
            elapsed_tokens += 1;
            if elapsed_tokens >= token {
                break;
            }
        }
        (line, token - token_to_last_line)
    }
}