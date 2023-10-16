use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use lalrpop_util::lalrpop_mod;
use shader_base::pass_id::PassID;
use shader_base::{AlphaMode, BindPoint, CompilationError, Culling, DescriptorType, FrontFace, PolygonMode, Property, ShaderInterface, ShaderParameters, ShaderStage, Topology};
use crate::ast::{HlslInstruction, Instruction};
use crate::hlsl_to_spirv::HlslToSpirv;
use crate::list_of::ListOf;
use crate::shader_pass::ShaderPass;

lalrpop_mod!(language);

mod ast;
mod list_of;
mod hlsl_to_spirv;
mod parsed_instructions;
mod shader_pass;

#[derive(Default, Clone)]
pub struct ReslShaderInterface {
    version: Option<u64>,
    blocks: HashMap<ShaderStage, HashMap<PassID, ShaderPass>>,
    errors: Vec<CompilationError>,
    parameters: ShaderParameters,
    file_path: PathBuf,
    bindings: HashMap<BindPoint, (DescriptorType, u32, HashSet<PassID>)>
}

impl ReslShaderInterface {
    fn parse_resl_code(resl_code: &str) -> Result<ListOf<Instruction>, CompilationError> {
        match language::InstructionListParser::new().parse(resl_code) {
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

    fn push_property(&mut self, name: &str, value: &str) -> Result<(), String> {
        let name = name.to_uppercase();
        let value = value.to_uppercase();
        match name.as_str() {
            "DEPTH_TEST" => {
                self.parameters.depth_test = match bool::from_str(value.to_lowercase().as_str()) {
                    Ok(i) => { i }
                    Err(_) => {
                        return Err(format!("DEPTH_TEST should be a bool : {}", value.to_lowercase()));
                    }
                };
            }
            "LINE_WIDTH" => {
                self.parameters.line_width = match f32::from_str(value.as_str()) {
                    Ok(i) => { i }
                    Err(_) => {
                        return Err(format!("LINE_WIDTH should be a float : {}", value));
                    }
                };
            }
            "SHADER_VERSION" => {
                self.version = Some(match u64::from_str(value.as_str()) {
                    Ok(i) => { i }
                    Err(_) => {
                        return Err(format!("SHADER_VERSION should be an unsigned int : {}", value));
                    }
                });
            }
            "ALPHA_MODE" => {
                self.parameters.alpha_mode = match value.as_str() {
                    "OPAQUE" => { AlphaMode::Opaque }
                    "TRANSLUCENT" => { AlphaMode::Translucent }
                    "ADDITIVE" => { AlphaMode::Additive }
                    _ => { return Err(format!("Wrong value : '{}', expected [OPAQUE, TRANSLUCENT, ADDITIVE]", value)); }
                };
            }
            "POLYGON" => {
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
            "FRONT" => {
                self.parameters.front_face = match value.as_str() {
                    "CLOCKWISE" => { FrontFace::Clockwise }
                    "COUNTER_CLOCKWISE" => { FrontFace::Clockwise }
                    _ => { return Err(format!("Wrong value : '{}', expected [CLOCKWISE, COUNTER_CLOCKWISE]", value)); }
                };
            }
            "CULL" => {
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

    fn push_block(&mut self, shader_stage: &ShaderStage, pass: PassID, content: ListOf<HlslInstruction>) -> Result<(), CompilationError> {
        match self.blocks.get_mut(shader_stage) {
            None => {
                let mut new_pass = ShaderPass::new(shader_stage.clone());
                new_pass.push_block(content)?;
                self.blocks.insert(shader_stage.clone(), HashMap::from([(pass, new_pass)]));
            }
            Some(stage) => {
                match stage.get_mut(&pass) {
                    None => {
                        let mut new_pass = ShaderPass::new(shader_stage.clone());
                        new_pass.push_block(content)?;
                        stage.insert(pass, new_pass);
                    }
                    Some(pass) => {
                        pass.push_block(content)?;
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
                    file_path
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
            file_path,
            blocks: Default::default(),
            errors: vec![],
            parameters: Default::default(),
            bindings: Default::default(),
        };

        let code = match parse_result {
            Ok(code) => { code }
            Err(err) => {
                interface.errors.push(err);
                ListOf::new()
            }
        };

        for instruction in code.iter() {
            if let Instruction::Block(_, stage, _, _) = instruction {
                interface.blocks.insert(stage.clone(), Default::default());
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
                    let mut keys = vec![];
                    for key in interface.blocks.keys() { keys.push(key.clone()) }
                    for stage in keys {
                        for render_pass in render_pass_group.iter() {
                            match &interface.push_block(&stage, PassID::new(render_pass), content.clone()) {
                                Ok(_) => {}
                                Err(err) => {
                                    interface.errors.push(if err.token.is_some() { err.clone() } else { CompilationError::throw(err.message.clone(), Some(*token)) })
                                }
                            }
                        }
                    }
                }
                Instruction::Block(token, stage, render_pass_group, content) => {
                    for render_pass in render_pass_group.iter() {
                        match interface.push_block(stage, PassID::new(render_pass), content.clone()) {
                            Ok(_) => {}
                            Err(err) => { interface.errors.push(if err.token.is_some() { err.clone() } else { CompilationError::throw(err.message.clone(), Some(*token)) }) }
                        }
                    }
                }
            }
        }

        interface
    }
}

impl ShaderInterface for ReslShaderInterface {
    fn get_spirv_for(&self, pass: &PassID, stage: &ShaderStage) -> Result<Vec<u32>, CompilationError> {
        let spirv = match self.blocks.get(stage) {
            None => { return Err(CompilationError::throw(format!("This shader is not available for stage {:?}", stage), None)); }
            Some(shader_stage) => {
                match shader_stage.get(pass) {
                    None => { return Err(CompilationError::throw(format!("This shader is not available for pass {pass}"), None)); }
                    Some(pass) => {
                        HlslToSpirv::default().transpile(&pass.get_text(), pass.entry_point_name(), &self.file_path, stage)
                    }
                }
            }
        };
        spirv
    }
    fn get_parameters_for(&self, _: &PassID) -> &ShaderParameters {
        &self.parameters
    }

    fn get_stage_inputs(&self, render_pass: &PassID, stage: &ShaderStage) -> Result<Vec<Property>, String> {
        match self.blocks.get(stage) {
            None => { return Err(format!("Shader stage ${:?} does not exists in the current shaders", stage)); }
            Some(stage) => {
                match stage.get(render_pass) {
                    None => { return Err(format!("Shader pass ${} does not exists in the current shaders", render_pass)); }
                    Some(block) => { block.stage_inputs() }
                }
            }
        }
    }

    fn get_stage_outputs(&self, render_pass: &PassID, stage: &ShaderStage) -> Result<Vec<Property>, String> {
        match self.blocks.get(stage) {
            None => { return Err(format!("Shader stage ${:?} does not exists in the current shaders", stage)); }
            Some(stage) => {
                match stage.get(render_pass) {
                    None => { return Err(format!("Shader pass ${} does not exists in the current shaders", render_pass)); }
                    Some(block) => { block.stage_output() }
                }
            }
        }
    }

    fn get_errors(&self) -> &Vec<CompilationError> {
        &self.errors
    }

    fn get_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn get_bindings(&self) -> HashMap<BindPoint, (DescriptorType, u32, HashSet<PassID>)> {
        
    }
}

#[test]
fn parse_resl() {
    let crate_path = "crates/engine/common/resl/";
    let file_path = "src/shader.resl";
    let _absolute_path = std::path::PathBuf::from(crate_path.to_string() + file_path);

    let _code = std::fs::read_to_string(file_path).unwrap();
}