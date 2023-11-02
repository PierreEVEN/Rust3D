use std::collections::HashMap;
use std::path::PathBuf;

use shader_base::{CompilationError, Property, ShaderResourcePool, ShaderStage};
use shader_base::pass_id::PassID;
use shader_base::spirv_reflector::SpirvReflector;
use shader_base::types::PixelFormat;

use crate::ast::{Function, FunctionParameter, HlslCodeBlock, HlslInstruction, HlslType, HlslTypeSimple, StructureField, Value};
use crate::hlsl_to_spirv::HlslToSpirv;
use crate::list_of::ListOf;
use crate::parsed_instructions::ParsedInstructions;

impl From<&HlslType> for PixelFormat {
    fn from(value: &HlslType) -> Self {
        match value {
            HlslType::Simple(t) => {
                match t {
                    HlslTypeSimple::Byte => { PixelFormat::R8_UNORM }
                    HlslTypeSimple::Int => { PixelFormat::R32_SINT }
                    HlslTypeSimple::Uint => { PixelFormat::R32_UINT }
                    HlslTypeSimple::Half => { PixelFormat::R16_SFLOAT }
                    HlslTypeSimple::Float => { PixelFormat::R32_SFLOAT }
                    HlslTypeSimple::Double => { PixelFormat::R64_SFLOAT }
                    _ => { PixelFormat::UNDEFINED }
                }
            }
            HlslType::Vec(t, u) => {
                match t {
                    HlslTypeSimple::Byte => {
                        match u {
                            1 => PixelFormat::R8_UNORM,
                            2 => PixelFormat::R8G8_UNORM,
                            3 => PixelFormat::R8G8B8_UNORM,
                            4 => PixelFormat::R8G8B8A8_UNORM,
                            _ => { PixelFormat::UNDEFINED }
                        }
                    }
                    HlslTypeSimple::Uint => {
                        match u {
                            1 => PixelFormat::R32_UINT,
                            2 => PixelFormat::R32G32_UINT,
                            3 => PixelFormat::R32G32B32_UINT,
                            4 => PixelFormat::R32G32B32A32_UINT,
                            _ => { PixelFormat::UNDEFINED }
                        }
                    }
                    HlslTypeSimple::Int => {
                        match u {
                            1 => PixelFormat::R32_SINT,
                            2 => PixelFormat::R32G32_SINT,
                            3 => PixelFormat::R32G32B32_SINT,
                            4 => PixelFormat::R32G32B32A32_SINT,
                            _ => { PixelFormat::UNDEFINED }
                        }
                    }
                    HlslTypeSimple::Half => {
                        match u {
                            1 => PixelFormat::R16_SFLOAT,
                            2 => PixelFormat::R16G16_SFLOAT,
                            3 => PixelFormat::R16G16B16_SFLOAT,
                            4 => PixelFormat::R16G16B16A16_SFLOAT,
                            _ => { PixelFormat::UNDEFINED }
                        }
                    }
                    HlslTypeSimple::Float => {
                        match u {
                            1 => PixelFormat::R32_SFLOAT,
                            2 => PixelFormat::R32G32_SFLOAT,
                            3 => PixelFormat::R32G32B32_SFLOAT,
                            4 => PixelFormat::R32G32B32A32_SFLOAT,
                            _ => { PixelFormat::UNDEFINED }
                        }
                    }
                    HlslTypeSimple::Double => {
                        match u {
                            1 => PixelFormat::R64_SFLOAT,
                            2 => PixelFormat::R64G64_SFLOAT,
                            3 => PixelFormat::R64G64B64_SFLOAT,
                            4 => PixelFormat::R64G64B64A64_SFLOAT,
                            _ => { PixelFormat::UNDEFINED }
                        }
                    }
                    _ => { PixelFormat::UNDEFINED }
                }
            }
            HlslType::Mat(_, _, _) => { PixelFormat::UNDEFINED }
            HlslType::Template(_, _) => { PixelFormat::UNDEFINED }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderPass {
    stage: ShaderStage,
    pass: PassID,
    instructions: ParsedInstructions,
    pragmas: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    structures: HashMap<String, ListOf<StructureField>>,
    properties: HashMap<String, HlslType>,
    auto_bound_resources: Vec<String>,
    entry_point: String,
    push_constant: Option<HlslType>,
    push_constant_size: Option<u32>,
    byte_code: Option<Vec<u32>>,
}

impl ShaderPass {
    pub fn new(stage: ShaderStage, pass: PassID) -> Self {
        Self {
            stage,
            pass,
            instructions: Default::default(),
            pragmas: Default::default(),
            structures: Default::default(),
            functions: Default::default(),
            properties: Default::default(),
            auto_bound_resources: vec![],
            entry_point: "main".to_string(),
            push_constant: None,
            push_constant_size: None,
            byte_code: None,
        }
    }

    // Add parsed hlsl code to this stage.
    pub fn push_block(&mut self, mut block: ListOf<HlslInstruction>) -> Result<(), CompilationError> {

        // Store and write pragma declarations first
        for instruction in block.iter() {
            if let HlslInstruction::Pragma(t, key, value) = instruction {
                if self.pragmas.contains_key(key) {
                    return Err(CompilationError::throw(format!("Pragma redeclaration : '{key}'"), Some(*t)));
                }
                if key.as_str() == "entry" {
                    match value {
                        Value::String(s) => { self.entry_point = s.clone(); }
                        _ => { return Err(CompilationError::throw(format!("#pragma entry should be of type String : found '{}'", value), Some(*t))); }
                    }
                }

                self.pragmas.insert(key.clone(), value.clone());
                self.instructions += (*t, "\n");
                self.instructions += (*t, format!("#pragma {key} {}", value));
                self.instructions += (*t, "\n");
            }
        }

        // We search for the entry point to fill the missing System Variables in the input and output structures
        let mut struct_with_sv_target: Option<String> = None;
        let mut struct_with_sv_position: Option<String> = None;
        for instruction in block.iter_mut() {
            if let HlslInstruction::Function(t, name, function) = instruction {
                if *name == self.entry_point {
                    match self.stage {
                        // Add SV_Target to all fragment outputs
                        ShaderStage::Fragment => {
                            match &function.return_type.1 {
                                HlslType::Simple(HlslTypeSimple::Struct(struct_name)) => {
                                    struct_with_sv_target = Some(struct_name.clone())
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
                                    if *s != HlslTypeSimple::Float || (*len != 4) {
                                        return Err(CompilationError::throw("Vertex stage support only float4 output".to_string(), Some(*t)));
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
                        for (target_id, field) in fields.iter_mut().enumerate() {
                            field.attribute = Some(format!("SV_Target{target_id}"));
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
                            return Err(CompilationError::throw("SV_POSITION is not defined for vertex stage output".to_string(), Some(*s)));
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

                    if let HlslType::Simple(HlslTypeSimple::ResourceImage) = ty {
                        self.auto_bound_resources.push(name.clone());
                    }

                    if let HlslType::Template(HlslTypeSimple::PushConstant, u) = ty {
                        if u.len() != 1 {
                            return Err(CompilationError::throw(format!("Push constant has more than one parameter : ${:?}", u), Some(*t)));
                        }
                        let ty = u.get(0).unwrap().clone();
                        self.instructions += (*t, format!("[[vk::push_constant]] ConstantBuffer<{}> {name};", ty));
                        self.push_constant = Some(ty);
                    } else {
                        self.instructions += (*t, format!("{} {name};", ty));
                    }
                }
                _ => {}
            }
        }


        Ok(())
    }

    pub fn compile_bytecode(&mut self, resources: &mut ShaderResourcePool, file_path: &PathBuf) -> Result<(), CompilationError> {
        let compiled = HlslToSpirv::default().transpile(&self.get_text(), self.entry_point_name(), &file_path, &self.stage)?;

        let compilation_result = match SpirvReflector::new(&compiled) {
            Ok(result) => { result }
            Err(err) => {
                return Err(CompilationError::throw(err, None));
            }
        };

        for (bp, binding) in &compilation_result.bindings {
            resources.add_resource(bp, binding.descriptor_type.clone(), &self.stage, &self.pass, binding.binding)?;
        }
        self.push_constant_size = compilation_result.push_constant_size;
        self.byte_code = Some(compiled);
        Ok(())
    }

    pub fn byte_code(&self) -> &Option<Vec<u32>> {
        &self.byte_code
    }

    pub fn entry_point_name(&self) -> &String {
        &self.entry_point
    }

    pub fn stage_inputs(&self) -> Result<Vec<Property>, String> {
        let entry_point = self.entry_point_name();
        let entry_func = match self.functions.get(entry_point.as_str()) {
            None => { return Err(format!("No entry point defined : expected '{entry_point}'")); }
            Some(func) => { func }
        };
        let mut properties = vec![];
        for prop in entry_func.params.iter() {
            if prop.attribute.is_some() {
                continue; // Skip special attributes (internal HLSL properties)
            }

            if let HlslType::Simple(HlslTypeSimple::Struct(str_name)) = &prop.param_type.1 {
                if let Some(str) = self.structures.get(str_name) {
                    for field in str.iter() {
                        let format = PixelFormat::from(&field.struct_type);
                        if format == PixelFormat::UNDEFINED {
                            return Err(format!("Cannot deduce pixel format for stage input property '{}' : {}", field.name, field.struct_type));
                        }
                        properties.push(Property {
                            name: field.name.clone(),
                            format,
                        })
                    }
                } else {
                    return Err(format!("Unknown structure '{}'", str_name));
                }
            } else {
                let format = PixelFormat::from(&prop.param_type.1);
                if format == PixelFormat::UNDEFINED {
                    return Err(format!("Cannot deduce pixel format for stage input property '{}' : {}", prop.name.1, prop.param_type.1));
                }
                properties.push(Property {
                    name: prop.name.1.clone(),
                    format,
                })
            }
        }
        Ok(properties)
    }

    pub fn stage_output(&self) -> Result<Vec<Property>, String> {
        let entry_point = self.entry_point_name();
        let entry_func = match self.functions.get(entry_point.as_str()) {
            None => { return Err(format!("No entry point defined : expected '{entry_point}'")); }
            Some(func) => { func }
        };

        let mut properties = vec![];
        for ty in self.unwrap_struct_type(&entry_func.return_type.1)? {
            let format = PixelFormat::from(&ty);
            if format == PixelFormat::UNDEFINED {
                return Err(format!("Cannot deduce pixel format for stage output property {} : {}", "return".to_string(), &ty));
            }
            properties.push(Property {
                name: "return".to_string(),
                format,
            })
        }
        Ok(properties)
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
            if let Some(attr) = &param.attribute {
                instr += (attr.0, format!(":{}", attr.1))
            }
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

    #[allow(dead_code)]
    pub fn get_token_from_location(&self, line: usize, column: usize) -> Option<usize> {
        self.instructions.get_token_for(line, column)
    }

    pub fn unwrap_struct_type(&self, ty: &HlslType) -> Result<Vec<HlslType>, String> {
        if let HlslType::Simple(HlslTypeSimple::Struct(s)) = ty {
            let mut types = vec![];

            for field in match self.structures.get(s) {
                None => { return Err(format!("Failed to find structure {s}")); }
                Some(str) => { str }
            }.iter() {
                types.append(&mut self.unwrap_struct_type(&field.struct_type)?);
            }
            Ok(types)
        } else {
            Ok(vec![ty.clone()])
        }
    }

    pub fn push_constant_size(&self) -> &Option<u32> {
        &self.push_constant_size
    }
}