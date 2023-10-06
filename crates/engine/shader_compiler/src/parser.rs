use crate::file_iterator::FileIterator;
use crate::includer::Includer;
use crate::types::ShaderErrorResult;
use gfx::shader::{ ShaderProperties };
use std::collections::HashMap;
use shader_base::pass_id::PassID;
use shader_base::{AlphaMode, Culling, FrontFace, PolygonMode, ShaderLanguage, ShaderStage, Topology};

#[derive(Default)]
pub struct ProgramData {
    chunks: HashMap<ShaderStage, HashMap<PassID, Vec<ShaderChunk>>>,
}

impl ProgramData {
    pub fn push_chunk(&mut self, pass: &PassID, stage: &ShaderStage, chunk: ShaderChunk) {
        match self.chunks.get_mut(stage) {
            None => {
                self.chunks
                    .insert(stage.clone(), HashMap::from([(pass.clone(), vec![chunk])]));
            }
            Some(passes) => match passes.get_mut(pass) {
                None => {
                    passes.insert(pass.clone(), vec![chunk]);
                }
                Some(chunks) => {
                    chunks.push(chunk);
                }
            },
        }
    }

    pub fn get_data(
        &self,
        pass: &PassID,
        stage: &ShaderStage,
    ) -> Result<&Vec<ShaderChunk>, ShaderErrorResult> {
        let mut errors = ShaderErrorResult::default();
        match self.chunks.get(stage) {
            None => {
                errors.push(
                    None,
                    None,
                    "ProgramData::get_data",
                    format!("failed to find pass {pass}").as_str(),
                    "",
                );
                Err(errors)
            }
            Some(passes) => match passes.get(pass) {
                None => {
                    errors.push(
                        None,
                        None,
                        "ProgramData::get_data",
                        format!("failed to find pass {pass} for stage {:?}", stage).as_str(),
                        "",
                    );
                    Err(errors)
                }
                Some(chunks) => Ok(chunks),
            },
        }
    }

    pub fn get_available_passes(&self) -> Vec<PassID> {
        let mut result = Vec::<PassID>::new();
        for value in self.chunks.values() {
            for key in value.keys() {
                if !result.contains(key) {
                    result.push(key.clone());
                }
            }
        }
        result
    }
}

pub struct Parser {
    file_iterator: FileIterator,
    includer: Box<dyn Includer>,
    internal_properties: HashMap<String, String>,
    pub properties: ShaderProperties,
    pub default_values: HashMap<String, String>,
    pub program_data: ProgramData,
}

#[derive(Clone, Default)]
pub struct ShaderChunk {
    pub virtual_path: String,
    pub line_start: u32,
    pub content: String,
}

impl Parser {
    fn skip_comment(&mut self) {
        if self.file_iterator.current() == '/' && &self.file_iterator + 1 == '/' {
            self.file_iterator.get_next_line();
        }
        if self.file_iterator.current() == '/' && &self.file_iterator + 1 == '*' {
            while self.file_iterator.valid()
                && !(self.file_iterator.current() == '*' && &self.file_iterator + 1 == '/')
            {
                self.file_iterator += 1;
            }
            self.file_iterator += 2;
        }
    }

    fn get_next_definition(&mut self) -> String {
        let mut line = String::new();
        while self.file_iterator.valid()
            && !(self.file_iterator.match_string_in_place("[")
                || self.file_iterator.match_string_in_place("=>"))
        {
            self.skip_comment();
            line.push(self.file_iterator.current());
            self.file_iterator += 1;
        }
        line
    }

    fn get_next_chunk(&mut self, virtual_path: &str) -> Result<ShaderChunk, ShaderErrorResult> {
        let mut current_indentation: i64 = 0;
        let mut found_body = false;
        let mut is_init = false;
        let mut chunk = ShaderChunk::default();
        let mut error = ShaderErrorResult::default();

        while self.file_iterator.valid() && (!found_body || current_indentation > 0) {
            self.skip_comment();

            // File inclusion
            if !found_body && self.file_iterator.match_string("=>") {
                self.file_iterator += 2;
                found_body = true;
                let file = Self::trim_string(&self.file_iterator.get_next_line());
                let result = (*self.includer).include_local(&file, virtual_path);
                match result {
                    Ok((name, data)) => {
                        chunk.line_start = self.file_iterator.current_line() as u32;
                        chunk.content = data;
                        chunk.virtual_path = name;
                        self.includer.release_include(&file, virtual_path);
                    }
                    Err(new_error) => {
                        self.includer.release_include(&file, virtual_path);
                        error += new_error;
                    }
                }
                continue;
            }
            // Chunk start
            else if self.file_iterator.match_string("[") {
                if current_indentation != 0 {
                    chunk.content.push('[');
                }
                current_indentation += 1;
                found_body = true;
            }
            // Chunk end
            else if self.file_iterator.match_string("]") {
                if current_indentation != 1 {
                    chunk.content.push(']');
                }
                current_indentation -= 1;
            }
            // Failed to find chunk limits
            else {
                if !found_body && !Self::is_void(self.file_iterator.current()) {
                    error.push(
                        Some(self.file_iterator.current_line() as isize),
                        None,
                        "Parser::get_next_chunk",
                        format!(
                            "expected '[' but found {}",
                            self.file_iterator.get_next_line()
                        )
                        .as_str(),
                        virtual_path,
                    );
                    break;
                }

                if !is_init {
                    is_init = true;
                    chunk.line_start = self.file_iterator.current_line() as u32;
                }

                chunk.content.push(self.file_iterator.current());
            }

            self.file_iterator += 1;
        }

        // No end
        if current_indentation != 0 {
            error.push(
                Some(self.file_iterator.current_line() as isize),
                None,
                "Parser::get_next_chunk",
                format!("chunk doesn't end correctly : {}", chunk.content).as_str(),
                virtual_path,
            );
        }
        // No body
        if !found_body {
            error.push(
                Some(self.file_iterator.current_line() as isize),
                None,
                "Parser::get_next_chunk",
                "failed to find chunk body",
                virtual_path,
            );
        }

        if error.empty() {
            return Ok(chunk);
        }
        Err(error)
    }

    fn property_trim_func(chr: char) -> bool {
        chr == ';'
            || chr == '='
            || chr == '\t'
            || chr == '\n'
            || chr == '\r'
            || chr == ' '
            || chr == '\''
            || chr == '"'
            || chr == ','
            || chr == '('
            || chr == ')'
    }

    fn trim_string(string: &String) -> String {
        let mut result = String::new();
        let mut begin: i32 = -1;
        let mut end: i32 = -1;
        let chars = string.chars();
        for i in 0..string.len() {
            if begin < 0 && !Self::property_trim_func(chars.clone().nth(i).unwrap()) {
                begin = i as i32;
                break;
            }
        }
        for i in (0..string.len()).rev() {
            if end < 0 && !Self::property_trim_func(chars.clone().nth(i).unwrap()) {
                end = i as i32;
                break;
            }
        }

        if begin >= 0 && end >= 0 {
            for i in begin..(end + 1) {
                result.push(chars.clone().nth(i as usize).unwrap());
            }
        }

        result
    }

    fn is_void(chr: char) -> bool {
        chr == ' ' || chr == '\t' || chr == '\r' || chr == '\n'
    }

    fn split_string(string: &str, separators: &[char]) -> Vec<String> {
        let mut content = Vec::<String>::new();
        let mut current_string = String::new();
        for chr in string.chars() {
            if separators.contains(&chr) && !current_string.is_empty() {
                content.push(current_string);
                current_string = String::new();
            } else {
                current_string.push(chr);
            }
        }
        if content.is_empty() || !current_string.is_empty() {
            content.push(current_string);
        }
        content
    }

    fn parse_head(header: &str) -> Result<Vec<(String, String)>, ShaderErrorResult> {
        let mut fields = Vec::new();
        let mut errors = ShaderErrorResult::default();
        let head_lines = Self::split_string(header, &[';']);
        for (i, head_line) in head_lines.iter().enumerate() {
            let prop_field = Self::split_string(head_line, &['=']);
            if prop_field.len() != 2 {
                errors.push(
                    Some(i as isize),
                    None,
                    "Parser::parse_head",
                    "syntax error",
                    "",
                );
                continue;
            }

            fields.push((
                Self::trim_string(&prop_field[0]),
                Self::trim_string(&prop_field[1]),
            ));
        }
        if !errors.empty() {
            return Err(errors);
        }
        Ok(fields)
    }

    fn parse_chunk_head(header: &str) -> Vec<PassID> {
        let mut passes = Vec::new();
        for field in Self::split_string(header, &[',']) {
            passes.push(PassID::new(Self::trim_string(&field).as_str()));
        }
        passes
    }

    fn get_property(&self, field: &str) -> &str {
        match self.internal_properties.get(field) {
            None => "",
            Some(property) => property.as_str(),
        }
    }

    pub fn new(
        shader_code: &str,
        file_path: &str,
        includer: Box<dyn Includer>,
    ) -> Result<Self, ShaderErrorResult> {
        let mut errors = ShaderErrorResult::default();

        let mut parser = Self {
            file_iterator: FileIterator::new(shader_code),
            includer,
            internal_properties: HashMap::new(),
            properties: ShaderProperties::default(),
            default_values: HashMap::new(),
            program_data: ProgramData::default(),
        };

        match parser.parse_shader(file_path) {
            Ok(_) => {}
            Err(error) => errors += error,
        };

        if !errors.empty() {
            Err(errors)
        } else {
            Ok(parser)
        }
    }

    fn parse_shader(&mut self, file_path: &str) -> Result<(), ShaderErrorResult> {
        let mut errors = ShaderErrorResult::default();

        while self.file_iterator.valid() {
            self.skip_comment();

            if self.file_iterator.match_string("#pragma") {
                let pragma_directive = self.file_iterator.get_next_line();
                let fields = Self::split_string(&pragma_directive, &[' ', '\t']);

                let mut content = String::from("");

                for field in fields.iter().skip(1) {
                    if field.chars().any(|c| c.is_ascii_alphanumeric()) {
                        content = Self::trim_string(field);
                        break;
                    }
                }

                self.internal_properties.insert(
                    Self::trim_string(&fields[0].to_lowercase()),
                    if fields.len() < 2 {
                        String::default()
                    } else {
                        content.to_uppercase()
                    },
                );
            }

            if self.file_iterator.match_string("head") {
                match self.get_next_chunk(file_path) {
                    Ok(chunk) => match Self::parse_head(&chunk.content) {
                        Ok(head) => {
                            for (key, value) in head {
                                self.default_values.insert(key, value);
                            }
                        }
                        Err(error) => {
                            errors += error;
                            errors.push(
                                Some(self.file_iterator.current_line() as isize),
                                None,
                                "Parser::parse_shader",
                                "failed to parse header for 'head' chunk",
                                file_path,
                            )
                        }
                    },
                    Err(error) => {
                        errors += error;
                        errors.push(
                            Some(self.file_iterator.current_line() as isize),
                            None,
                            "Parser::parse_shader",
                            "failed to read 'head' chunk",
                            file_path,
                        );
                    }
                };
            }

            if self.file_iterator.match_string("global") {
                let global_args = self.get_next_definition();
                match self.get_next_chunk(file_path) {
                    Ok(chunk) => {
                        let mut chunk_data = chunk.clone();
                        if chunk_data.virtual_path.is_empty() {
                            chunk_data.virtual_path = file_path.to_string();
                        }
                        for pass in Self::parse_chunk_head(&global_args) {
                            self.program_data.push_chunk(
                                &pass,
                                &ShaderStage::Vertex,
                                chunk_data.clone(),
                            );
                            self.program_data.push_chunk(
                                &pass,
                                &ShaderStage::Fragment,
                                chunk_data.clone(),
                            );
                        }
                    }
                    Err(error) => {
                        errors += error;
                        errors.push(
                            Some(self.file_iterator.current_line() as isize),
                            None,
                            "Parser::parse_shader",
                            "failed to read 'global' chunk",
                            file_path,
                        );
                    }
                };
            }

            if self.file_iterator.match_string("vertex") {
                let vertex_args = self.get_next_definition();
                match self.get_next_chunk(file_path) {
                    Ok(vertex_chunk) => {
                        let mut chunk_data = vertex_chunk.clone();
                        if chunk_data.virtual_path.is_empty() {
                            chunk_data.virtual_path = file_path.to_string();
                        }
                        for pass in Self::parse_chunk_head(&vertex_args) {
                            self.program_data.push_chunk(
                                &pass,
                                &ShaderStage::Vertex,
                                chunk_data.clone(),
                            );
                        }
                    }
                    Err(error) => {
                        errors += error;
                        errors.push(
                            Some(self.file_iterator.current_line() as isize),
                            None,
                            "Parser::parse_shader",
                            "failed to read 'vertex' chunk",
                            file_path,
                        );
                    }
                };
            }

            if self.file_iterator.match_string("fragment") {
                let fragment_args = self.get_next_definition();
                match self.get_next_chunk(file_path) {
                    Ok(fragment_chunk) => {
                        let mut chunk_data = fragment_chunk.clone();
                        if chunk_data.virtual_path.is_empty() {
                            chunk_data.virtual_path = file_path.to_string();
                        }
                        for pass in Self::parse_chunk_head(&fragment_args) {
                            self.program_data.push_chunk(
                                &pass,
                                &ShaderStage::Fragment,
                                chunk_data.clone(),
                            );
                        }
                    }
                    Err(error) => {
                        errors += error;
                        errors.push(
                            Some(self.file_iterator.current_line() as isize),
                            None,
                            "Parser::parse_shader",
                            "failed to read 'fragment' chunk",
                            file_path,
                        );
                    }
                }
            }

            self.file_iterator += 1;
        }

        self.properties = ShaderProperties {
            shader_version: self.get_property("shader_version").to_string(),
            shader_language: match self.get_property("shader_language") {
                "GLSL" => ShaderLanguage::GLSL,
                "HLSL" => ShaderLanguage::HLSL,
                _ => ShaderLanguage::default(),
            },
            culling: match self.get_property("cull") {
                "FRONT" => Culling::Front,
                "BACK" => Culling::Back,
                "BOTH" => Culling::Both,
                "NONE" => Culling::None,
                _ => Culling::default(),
            },
            front_face: match self.get_property("front_face") {
                "CLOCKWISE" => FrontFace::Clockwise,
                "COUNTER_CLOCKWISE" => FrontFace::CounterClockwise,
                _ => FrontFace::default(),
            },
            topology: match self.get_property("topology") {
                "TRIANGLES" => Topology::Triangles,
                "POINTS" => Topology::Points,
                "LINES" => Topology::Lines,
                _ => Topology::default(),
            },
            polygon_mode: match self.get_property("polygon") {
                "FILL" => PolygonMode::Fill,
                "POINT" => PolygonMode::Point,
                "LINE" => PolygonMode::Line,
                _ => PolygonMode::default(),
            },
            alpha_mode: match self.get_property("alpha_mode") {
                "OPAQUE" => AlphaMode::Opaque,
                "TRANSLUCENT" => AlphaMode::Translucent,
                "ADDITIVE" => AlphaMode::Additive,
                _ => AlphaMode::default(),
            },
            depth_test: false,
            line_width: 0.0,
        };

        if errors.empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
