use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use lalrpop_util::lalrpop_mod;
use shader_base::pass_id::PassID;
use shader_base::{AlphaMode, CompilationError, Culling, FrontFace, PolygonMode, ShaderInterface, ShaderParameters, ShaderStage, Topology};
use crate::ast::{HlslInstruction, Instruction};
use crate::list_of::ListOf;

lalrpop_mod!(language);

mod ast;
mod list_of;
pub mod resl;
mod hlsl_to_spirv;

#[derive(Default)]
pub struct ReslShaderInterface {
    version: Option<u64>,
    _code: String,
    blocks: HashMap<ShaderStage, HashMap<PassID, Vec<Arc<ListOf<HlslInstruction>>>>>,
    errors: Vec<CompilationError>,
    parameters: ShaderParameters,
}

impl ReslShaderInterface {
    fn parse_resl_code(resl_code: &String) -> Result<ListOf<Instruction>, CompilationError> {
        match language::InstructionListParser::new().parse(resl_code.as_str()) {
            Ok(parsed) => Ok(parsed),
            Err(e) => match e {
                lalrpop_util::ParseError::UnrecognizedToken { token, expected } => {
                    let (start, found, _) = token;
                    Err(CompilationError {
                        message: format!("Unrecognized token {:?} : expected {:?}", found, expected),
                        token: Some(start),
                    })
                }
                lalrpop_util::ParseError::InvalidToken { location } => {
                    Err(CompilationError {
                        message: format!("Invalid token {:?}", resl_code.chars().nth(location).unwrap()),
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
        }
    }

    fn set_version(&mut self, version: &u64) -> Result<(), String> {
        match self.version {
            None => {
                self.version = Some(*version);
                Ok(())
            }
            Some(current) => {
                if *version != current {
                    Err(format!("Version already declared before : {} override {}", version, current))
                } else { Ok(()) }
            }
        }
    }

    fn push_property(&mut self, name: &String, value: &String) -> Result<(), String> {
        let name = name.to_uppercase();
        let value = value.to_uppercase();
        match name.as_str() {
            "ALPHA_MODE" => {
                self.parameters.alpha_mode = match value.as_str() {
                    "OPAQUE" => { AlphaMode::Opaque }
                    "TRANSLUCENT" => { AlphaMode::Translucent }
                    "ADDITIVE" => { AlphaMode::Additive }
                    _ => { return Err(format!("Wrong value : '{}', expected [OPAQUE, TRANSLUCENT, ADDITIVE]", value)); }
                };
            }
            "POLYGON_MODE" => {
                self.parameters.polygon_mode = match value.as_str() {
                    "FILL" => { PolygonMode::Fill }
                    "POINT" => { PolygonMode::Point }
                    "LINE" => { PolygonMode::Line }
                    _ => { return Err(format!("Wrong value : '{}', expected [FILL, POINT, LINE]", value)); }
                };
            }
            "TOPOLOGY" => {
                self.parameters.topology = match value.as_str() {
                    "LINES" => { Topology::Lines }
                    "POINTS" => { Topology::Points }
                    "TRIANGLES" => { Topology::Triangles }
                    _ => { return Err(format!("Wrong value : '{}', expected [LINES, POINTS, TRIANGLES]", value)); }
                };
            }
            "FRONT_FACE" => {
                self.parameters.front_face = match value.as_str() {
                    "CLOCKWISE" => { FrontFace::Clockwise }
                    "COUNTER_CLOCKWISE" => { FrontFace::Clockwise }
                    _ => { return Err(format!("Wrong value : '{}', expected [CLOCKWISE, COUNTER_CLOCKWISE]", value)); }
                };
            }
            "CULLING" => {
                self.parameters.culling = match value.as_str() {
                    "NONE" => { Culling::None }
                    "FRONT" => { Culling::Front }
                    "BACK" => { Culling::Back }
                    "BOTH" => { Culling::Both }
                    _ => { return Err(format!("Wrong value : '{}', expected [NONE, FRONT, BACK, BOTH]", value)); }
                };
            }
            _ => return Err(format!("Unknown property '{}'", name))
        }
        Ok(())
    }

    fn push_block(&mut self, stage: &ShaderStage, pass: PassID, content: Arc<ListOf<HlslInstruction>>) -> Result<(), String> {
        match self.blocks.get_mut(stage) {
            None => {
                self.blocks.insert(stage.clone(), HashMap::from([(pass, vec![content])]));
            }
            Some(stage) => {
                match stage.get_mut(&pass) {
                    None => {
                        stage.insert(pass, vec![content]);
                    }
                    Some(pass) => {
                        pass.push(content);
                    }
                }
            }
        }
        Ok(())
    }
}

impl From<PathBuf> for ReslShaderInterface {
    fn from(file_path: PathBuf) -> Self {
        let resl_code = match fs::read_to_string(file_path.clone()) {
            Ok(code) => { code }
            Err(error) => {
                let absolute_path = if file_path.is_absolute() {
                    file_path.to_path_buf()
                } else {
                    std::env::current_dir().unwrap().join(file_path)
                };
                logger::error!("Failed to open file {} : {}", absolute_path.to_str().unwrap(), error);
                return Self::default();
            }
        };

        let parse_result = ReslShaderInterface::parse_resl_code(&resl_code);

        let mut interface = Self {
            version: None,
            _code: resl_code,
            blocks: Default::default(),
            errors: vec![],
            parameters: Default::default(),
        };

        let code = match parse_result {
            Ok(code) => { code }
            Err(err) => {
                interface.errors.push(err);
                ListOf::new()
            }
        };

        for instruction in code.iter() {
            match instruction {
                Instruction::Block(_, stage, _, _) => {
                    interface.blocks.insert(stage.clone(), Default::default());
                }
                _ => {}
            }
        }

        for instruction in code.iter() {
            match instruction {
                Instruction::Version(token, version) => {
                    match interface.set_version(version) {
                        Ok(_) => {}
                        Err(err) => { interface.errors.push(CompilationError::throw(err, Some(*token))) }
                    }
                }
                Instruction::Pragma(token, key, value) => {
                    match interface.push_property(key, &format!("{}", value)) {
                        Ok(_) => {}
                        Err(err) => { interface.errors.push(CompilationError::throw(err, Some(*token))) }
                    }
                }
                Instruction::Global(token, render_pass_group, content) => {
                    let block = Arc::new((*content).clone());
                    let mut keys = vec![];
                    for key in interface.blocks.keys() { keys.push(key.clone()) }
                    for stage in keys {
                        for render_pass in render_pass_group.iter() {
                            match &interface.push_block(&stage, PassID::new(render_pass), block.clone()) {
                                Ok(_) => {}
                                Err(err) => { interface.errors.push(CompilationError::throw(err.clone(), Some(*token))) }
                            }
                        }
                    }
                }
                Instruction::Block(token, stage, render_pass_group, content) => {
                    let block = Arc::new((*content).clone());
                    for render_pass in render_pass_group.iter() {
                        match interface.push_block(stage, PassID::new(render_pass), block.clone()) {
                            Ok(_) => {}
                            Err(err) => { interface.errors.push(CompilationError::throw(err, Some(*token))) }
                        }
                    }
                }
            }
        }

        interface
    }
}

impl ShaderInterface for ReslShaderInterface {
    fn get_spirv_for(&self, _: &PassID, _: ShaderStage) -> Vec<u32> {
        todo!()
    }
    fn get_parameters_for(&self, _: &PassID) -> &ShaderParameters {
        &self.parameters
    }
}

#[test]
fn parse_resl() {
    let crate_path = "crates/engine/common/resl/";
    let file_path = "src/shader.resl";
    let absolute_path = std::path::PathBuf::from(crate_path.to_string() + file_path);

    let code = std::fs::read_to_string(file_path).unwrap();

    let mut builder = resl::Parser::default();


    match builder.parse(code, absolute_path.clone()) {
        Ok(_) => {}
        Err(err) => {
            match err.token {
                None => {
                    panic!("{}\n  --> {}", err.message, absolute_path.to_str().unwrap());
                }
                Some(token) => {
                    let (line, column) = builder.get_error_location(token);
                    panic!("{}\n  --> {}:{}:{}", err.message, absolute_path.to_str().unwrap(), line, column);
                }
            }
        }
    };

    println!("{:?}", builder.hlsl);
}