use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use lalrpop_util::lalrpop_mod;

use shader_base::{AlphaMode, CompilationError, Culling, FrontFace, PolygonMode, Property, ShaderInterface, ShaderParameters, ShaderResourcePool, ShaderStage, Topology};
use shader_base::pass_id::PassID;

use crate::ast::{HlslInstruction, Instruction};
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
    per_stage_data: HashMap<ShaderStage, HashMap<PassID, ShaderPass>>,
    errors: Vec<CompilationError>,
    parameters: ShaderParameters,
    file_path: PathBuf,
    resources: Arc<ShaderResourcePool>,
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
        match self.per_stage_data.get_mut(shader_stage) {
            None => {
                let mut new_pass = ShaderPass::new(shader_stage.clone(), pass.clone());
                new_pass.push_block(content)?;
                self.per_stage_data.insert(shader_stage.clone(), HashMap::from([(pass, new_pass)]));
            }
            Some(stage) => {
                match stage.get_mut(&pass) {
                    None => {
                        let mut new_pass = ShaderPass::new(shader_stage.clone(), pass.clone());
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

// Parse a RESL shader file from path
impl From<PathBuf> for ReslShaderInterface {
    fn from(file_path: PathBuf) -> Self {

        // Read file
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

        let mut interface = Self {
            version: None,
            file_path,
            per_stage_data: Default::default(),
            errors: vec![],
            parameters: Default::default(),
            resources: Default::default(),
        };

        // Parse file using LALRPOP
        let code = match ReslShaderInterface::parse_resl_code(&resl_code) {
            Ok(code) => { code }
            Err(err) => {
                interface.errors.push(err);
                ListOf::new()
            }
        };

        // Initialize all the available stages
        for instruction in code.iter() {
            if let Instruction::Block(_, stage, _, _) = instruction {
                interface.per_stage_data.insert(stage.clone(), Default::default());
            }
        }

        // Read each parsed field
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
                    for key in interface.per_stage_data.keys() { keys.push(key.clone()) }
                    // Push the global group in each stage of each found pass
                    for stage in keys {
                        for render_pass in render_pass_group.iter() {
                            if let Err(err) = &interface.push_block(&stage, PassID::new(render_pass), content.clone()) {
                                interface.errors.push(if err.token.is_some() { err.clone() } else { CompilationError::throw(err.message.clone(), Some(*token)) })
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
        
        let mut resources = Default::default();
        // Compile parsed code to SPIRV bytecode
        for stage in interface.per_stage_data.values_mut() {
            for pass in stage.values_mut() {
                if let Err(err) = pass.compile_bytecode(&mut resources, &interface.file_path) {
                    interface.errors.push(err);
                }
            }
        }
        interface.resources = Arc::new(resources);
        
        interface
    }
}

impl ShaderInterface for ReslShaderInterface {
    fn get_spirv_for(&self, pass: &PassID, stage: &ShaderStage) -> Result<Vec<u32>, CompilationError> {
        match self.per_stage_data.get(stage) {
            None => { Err(CompilationError::throw(format!("This shader is not available for stage {:?}", stage), None)) }
            Some(shader_stage) => {
                match shader_stage.get(pass) {
                    None => { Err(CompilationError::throw(format!("This shader is not available for pass {pass}"), None)) }
                    Some(pass) => {
                        if let Some(spirv) = pass.byte_code() {
                            Ok(spirv.clone())
                        } else {
                            Err(CompilationError::throw("No spirv code available".to_string(), None))
                        }
                    }
                }
            }
        }
    }
    fn get_parameters_for(&self, _: &PassID) -> &ShaderParameters {
        &self.parameters
    }

    fn get_stage_inputs(&self, render_pass: &PassID, stage: &ShaderStage) -> Result<Vec<Property>, String> {
        match self.per_stage_data.get(stage) {
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
        match self.per_stage_data.get(stage) {
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

    fn resource_pool(&self) -> &Arc<ShaderResourcePool> {
        &self.resources
    }

    fn get_entry_point(&self, render_pass: &PassID, stage: &ShaderStage) -> Result<String, String> {
        match self.per_stage_data.get(stage) {
            None => { return Err(format!("Shader stage ${:?} does not exists in the current shaders", stage)); }
            Some(stage) => {
                match stage.get(render_pass) {
                    None => { return Err(format!("Shader pass ${} does not exists in the current shaders", render_pass)); }
                    Some(block) => { Ok(block.entry_point_name().clone()) }
                }
            }
        }
    }

    fn push_constant_size(&self, stage: &ShaderStage, pass: &PassID) -> Option<u32> {
        if let Some(stage) = self.per_stage_data.get(stage) {
            if let Some(pass) = stage.get(pass) {
                return *pass.push_constant_size()
            }
        }
        None
    }
}

#[test]
fn parse_resl() {
    let crate_path = "crates/engine/common/resl/";
    let file_path = "src/shader.resl";
    let _absolute_path = std::path::PathBuf::from(crate_path.to_string() + file_path);

    let _code = std::fs::read_to_string(file_path).unwrap();
}