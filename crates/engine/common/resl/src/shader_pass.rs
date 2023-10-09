use std::collections::HashMap;
use shader_base::{CompilationError, ShaderStage};
use crate::ast::{Function, FunctionParameter, HlslCodeBlock, HlslInstruction, HlslType, HlslTypeSimple, StructureField, Value};
use crate::list_of::ListOf;
use crate::parsed_instructions::ParsedInstructions;

#[derive(Debug)]
pub struct ShaderPass {
    stage: ShaderStage,
    instructions: ParsedInstructions,
    pragmas: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    structures: HashMap<String, ListOf<StructureField>>,
    properties: HashMap<String, HlslType>,
    entry_point: String,
}

impl ShaderPass {
    pub fn new(stage: ShaderStage) -> Self {
        Self {
            stage,
            instructions: Default::default(),
            pragmas: Default::default(),
            functions: Default::default(),
            structures: Default::default(),
            properties: Default::default(),
            entry_point: "main".to_string(),
        }
    }

    // Add parsed hlsl code to this stage.
    pub fn push_block(&mut self, mut block: ListOf<HlslInstruction>) -> Result<(), CompilationError> {

        // Store and write pragma declarations first
        for instruction in block.iter() {
            match instruction {
                HlslInstruction::Pragma(t, key, value) => {
                    if self.pragmas.contains_key(key) {
                        return Err(CompilationError::throw(format!("Pragma redeclaration : '{key}'"), Some(*t)));
                    }
                    match key.as_str() {
                        "entry" => {
                            match value {
                                Value::String(s) => { self.entry_point = s.clone(); }
                                _ => { return Err(CompilationError::throw(format!("#pragma entry should be of type String : found '{}'", value.to_string()), Some(*t))); }
                            }
                        }
                        _ => {}
                    }

                    self.pragmas.insert(key.clone(), value.clone());
                    self.instructions += (*t, "\n");
                    self.instructions += (*t, format!("#pragma {key} {}", value));
                    self.instructions += (*t, "\n");
                }
                _ => {}
            }
        }

        // We search for the entry point to fill the missing System Variables in the input and output structures
        let mut struct_with_sv_target: Option<String> = None;
        let mut struct_with_sv_position: Option<String> = None;
        for instruction in (&mut block).iter_mut() {
            if let HlslInstruction::Function(t, name, function) = instruction {
                if *name == self.entry_point {
                    match self.stage {
                        // Add SV_Target to all fragment outputs
                        ShaderStage::Fragment => {
                            match &function.return_type.1 {
                                HlslType::Simple(s) => {
                                    match s {
                                        HlslTypeSimple::Struct(struct_name) => {
                                            struct_with_sv_target = Some(struct_name.clone());
                                        }
                                        _ => { function.attribute = Some((*t, "SV_Target".to_string())) }
                                    }
                                }
                                _ => { function.attribute = Some((*t, "SV_Target".to_string())) }
                            }
                        }
                        ShaderStage::Vertex => {
                            // Ensure SV_Position is set for vertex output
                            match &function.return_type.1 {
                                HlslType::Simple(s) => {
                                    if let HlslTypeSimple::Struct(struct_name) = s {
                                        struct_with_sv_position = Some(struct_name.clone());
                                    } else {
                                        return Err(CompilationError::throw(format!("{} is not a valid type for a vertex stage output", s.to_string()), Some(*t)));
                                    }
                                }
                                HlslType::Vec(s, len) => {
                                    if *s != HlslTypeSimple::Float || *len != 4 {
                                        return Err(CompilationError::throw(format!("Vertex stage support only float4 output"), Some(*t)));
                                    }
                                    function.attribute = Some((*t, "SV_POSITION".to_string()))
                                }
                                other => { return Err(CompilationError::throw(format!("{} is not a valid type for a vertex stage output", other.to_string()), Some(*t))); }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // Add SV_Target# to fields of fragment output
        if let Some(struct_name) = struct_with_sv_target {
            for instr_in in block.iter_mut() {
                if let HlslInstruction::Struct(_, name, fields) = instr_in {
                    if *name == struct_name {
                        let mut target_id = 0;
                        for field in fields.iter_mut() {
                            field.attribute = Some(format!("SV_Target{target_id}"));
                            target_id += 1;
                        }
                    }
                }
            }
        }
        // Ensure SV_Position is correctly set
        if let Some(struct_name) = struct_with_sv_position {
            for instr_in in block.iter() {
                if let HlslInstruction::Struct(s, name, fields) = instr_in {
                    if *name == struct_name {
                        let mut contains = false;
                        for field in fields.iter() {
                            if let Some(attr) = &field.attribute {
                                if attr.to_uppercase() == "SV_POSITION" { contains = true }
                            }
                        }
                        if !contains {
                            return Err(CompilationError::throw(format!("SV_POSITION is not defined for vertex stage output"), Some(*s)));
                        }
                    }
                }
            }
        }

        for instruction in block.iter() {
            match instruction {
                // struct name { type name; ... };
                HlslInstruction::Struct(t, name, fields) => {
                    if self.structures.contains_key(name) {
                        return Err(CompilationError::throw(format!("Structure redeclaration : '{name}'"), Some(*t)));
                    }
                    self.structures.insert(name.clone(), fields.clone());

                    self.instructions += (*t, format!("struct {name} {{"));
                    for field in fields.iter() {
                        self.instructions += (field.token, format!("{} {}", field.struct_type.to_string(), field.name));
                        if let Some(value) = &field.value {
                            self.instructions += (field.token, format!("={}", value))
                        }
                        if let Some(attribute) = &field.attribute {
                            self.instructions += (field.token, format!(":{}", attribute))
                        }
                        self.instructions += (field.token, ";");
                    }
                    self.instructions += (t, "};");
                }
                // #define key value
                HlslInstruction::Define(t, key, value) => {
                    self.instructions += (*t, "\n");
                    self.instructions += (*t, format!("#define {key}"));
                    if let Some(value) = &value {
                        self.instructions += (t, value);
                    }
                    self.instructions += (t, "\n")
                }
                HlslInstruction::Include(_, _) => { logger::warning!("Include are not handled yet !") }
                HlslInstruction::Function(t, name, function) => {
                    if self.functions.contains_key(name) {
                        return Err(CompilationError::throw(format!("Function redeclaration : '{name}'"), Some(*t)));
                    }
                    self.functions.insert(name.clone(), function.clone());
                    self.instructions += Self::parse_function(t, name, function)
                }
                HlslInstruction::Property(t, ty, name) => {
                    if self.properties.contains_key(name) {
                        return Err(CompilationError::throw(format!("Property redeclaration : '{name}'"), Some(*t)));
                    }
                    self.properties.insert(name.clone(), ty.clone());
                    self.instructions += (*t, format!("{} {name};", ty.to_string()));
                }
                _ => {}
            }
        }


        Ok(())
    }

    pub fn entry_point_name(&self) -> &String {
        &self.entry_point
    }

    pub fn stage_inputs(&self) -> Result<(), String> {
        let entry_point = self.entry_point_name();
        let entry_func = match self.functions.get(entry_point.as_str()) {
            None => { return Err(format!("No entry point defined : expected '{entry_point}'")); }
            Some(func) => { func }
        };
        let mut params: String;
        for param in entry_func.params.iter() {}
        todo!()
    }

    pub fn stage_output(&self) -> Result<HlslType, String> {
        let entry_point = self.entry_point_name();
        let entry_func = match self.functions.get(entry_point.as_str()) {
            None => { return Err(format!("No entry point defined : expected '{entry_point}'")); }
            Some(func) => { func }
        };
        Ok(entry_func.return_type.1.clone())
    }

    fn parse_block(block: &ListOf<HlslCodeBlock>) -> ParsedInstructions {
        let mut instructions = ParsedInstructions::default();
        let mut was_last_text = false;
        for inst in block.iter() {
            match inst {
                HlslCodeBlock::InnerBlock(begin, end, block) => {
                    instructions += (begin, "{");
                    Self::parse_block(block);
                    instructions += (end, "}");
                }
                HlslCodeBlock::Text(t, text) => {
                    if was_last_text { instructions += (*t, format!(" {text}")) } else { instructions += (t, text) }
                    was_last_text = true;
                    continue;
                }
                HlslCodeBlock::Token(t, tok) => { instructions += (*t, tok.to_string()) }
                HlslCodeBlock::Semicolon(t) => { instructions += (t, ";") }
            }
            was_last_text = false;
        }
        instructions
    }

    fn parse_function(token: &usize, name: &String, function: &Function) -> ParsedInstructions {
        let mut instructions = ParsedInstructions::default();


        instructions += (function.return_type.0, function.return_type.1.to_string());

        instructions += (*token, format!(" {name}("));

        let mut sep = ParsedInstructions::default();
        sep += (*token, ",");

        instructions += function.params.join::<ParsedInstructions, ParsedInstructions, _>(sep, |param: &FunctionParameter| -> ParsedInstructions {
            let mut instr = ParsedInstructions::default();
            instr += (param.param_type.0, param.param_type.1.to_string());
            instr += (param.name.0, format!(" {}", param.name.1));
            instr
        });

        instructions += (*token, ")");

        if let Some(attribute) = &function.attribute { instructions += (attribute.0, format!(":{}", attribute.1)); }
        let (start, end, content) = &function.content;
        instructions += (start, "{");
        instructions += Self::parse_block(content);
        instructions += (end, "}");
        instructions
    }


    pub fn get_text(&self) -> String {
        self.instructions.get_text()
    }

    pub fn get_token_from_location(&self, line: usize, column: usize) -> Option<usize> {
        self.instructions.get_token_for(line, column)
    }
}