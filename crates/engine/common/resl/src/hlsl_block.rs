use std::fmt::format;
use crate::ast::{Function, FunctionParameter, HlslCodeBlock, HlslInstruction};
use crate::list_of::ListOf;
use crate::parsed_instructions::ParsedInstructions;

pub struct HlslBlock {
    instructions: ParsedInstructions,
}

impl HlslBlock {
    pub fn new(instructions: ListOf<HlslInstruction>) -> Self {
        Self { instructions: Self::parse_instructions(&instructions) }
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
                    if was_last_text {instructions += (*t, format!(" {text}"))}
                    else {instructions += (t, text)}
                    was_last_text = true;
                    continue;
                }
                HlslCodeBlock::Token(t, tok) => {instructions += (*t, tok.to_string())}
                HlslCodeBlock::Semicolon(t) => {instructions += (t, ";")}
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

    fn parse_instructions(instructions: &ListOf<HlslInstruction>) -> ParsedInstructions {
        let mut instr = ParsedInstructions::default();

        for instruction in instructions.iter() {
            match instruction {
                HlslInstruction::Struct(t, name, register, fields) => {
                    if register.is_some() {
                        logger::warning!("Todo : remove register from supported syntax");
                    }
                    instr += (*t, format!("struct {name} {{"));
                    for field in fields.iter() {
                        instr += (field.token, format!("{} {}", field.struct_type.to_string(), field.name));
                        if let Some(value) = &field.value {
                            instr += (field.token, format!("={}", value))
                        }
                        if let Some(attribute) = &field.attribute {
                            instr += (field.token, format!(":{}", attribute))
                        }
                        instr += (field.token, ";");
                    }
                    instr += (t, "};");
                }
                HlslInstruction::Define(t, key, value) => {
                    instr += (*t, "\n");
                    instr += (*t, format!("#define {key}"));
                    if let Some(value) = &value {
                        instr += (t, value);
                    }
                    instr += (t, "\n")
                }
                HlslInstruction::Include(_, _) => { logger::warning!("Include are not handled yet !") }
                HlslInstruction::Function(t, name, function) => { instr += Self::parse_function(t, name, function) }
                HlslInstruction::Property(t, ty, name, val) => {
                    if val.is_some() { logger::warning!("Todo : Remove register from supported language features") }
                    instr += (*t, format!("{} {name}", ty.to_string()));
                }
                HlslInstruction::Pragma(t, key, value) => {
                    instr += (*t, "\n");
                    instr += (*t, format!("#pragma {key} {}", value));
                    instr += (*t, "\n");
                }
            }
        }


        instr
    }

    pub fn get_text(&self) -> String {
        self.instructions.get_text()
    }

    pub fn get_token_from_location(&self, line: usize, column: usize) -> Option<usize> {
        self.instructions.get_token_for(line, column)
    }
}