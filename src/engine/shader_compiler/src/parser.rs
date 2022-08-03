﻿use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::backends::backend_glslang::GlslangIncluder;
use crate::file_iterator::FileIterator;
use crate::includer::Includer;
use crate::ShaderLanguage;
use crate::types::{AlphaMode, Culling, FrontFace, PolygonMode, ShaderErrorResult, ShaderProperties, Topology};

#[derive(Default)]
pub struct ShaderPass
{
    pass_name: String,
    vertex_chunks: Vec<ShaderChunk>,
    fragment_chunks: Vec<ShaderChunk>,
}

pub struct Parser {
    file_iterator: FileIterator,
    includer: Box<dyn Includer>,
    internal_properties: HashMap::<String, String>,
    pub properties: ShaderProperties,
    pub globals: Vec::<ShaderChunk>,
    pub default_values: HashMap<String, String>,
    pub passes: HashMap<String, ShaderPass>,
}

#[derive(Clone, Default)]
pub struct ShaderChunk {
    pub file: String,
    pub line_start: u32,
    pub content: String,
}

impl Parser {
    fn skip_comment(&mut self) {
        let next = &self.file_iterator + 1;

        if self.file_iterator.current() == '/' && &self.file_iterator + 1 == '/' {
            self.file_iterator.get_next_line();
        }
        if self.file_iterator.current() == '/' && &self.file_iterator + 1 == '*'
        {
            while self.file_iterator.valid() && !(self.file_iterator.current() == '*' && &self.file_iterator + 1 == '/') {
                self.file_iterator += 1;
            }
            self.file_iterator += 2;
        }
    }

    fn get_next_definition(&mut self) -> String
    {
        let mut line = String::new();
        while self.file_iterator.valid() && !(self.file_iterator.match_string_in_place("[") || self.file_iterator.match_string_in_place("=>"))
        {
            self.skip_comment();
            line.push(self.file_iterator.current());
            self.file_iterator += 1;
        }
        return line;
    }

    fn get_next_chunk(&mut self, file_path: &Path) -> Result<ShaderChunk, ShaderErrorResult>
    {
        let mut current_indentation: i64 = 0;
        let mut found_body = false;
        let mut is_init = false;
        let mut chunk = ShaderChunk::default();
        let mut error = ShaderErrorResult::default();

        while self.file_iterator.valid() && (!found_body || current_indentation > 0)
        {
            self.skip_comment();

            // File inclusion            
            if self.file_iterator.match_string("=>")
            {
                self.file_iterator += 2;
                found_body = true;
                let file = Self::trim_string(&self.file_iterator.get_next_line());
                let result = (*self.includer).include_local(&file, &file_path);
                match result {
                    Ok((name, data)) => {
                        chunk.line_start = 0;
                        chunk.content = data;
                        chunk.file = name;
                        self.includer.release_include(&file, &file_path);
                    }
                    Err(new_error) => {
                        self.includer.release_include(&file, &file_path);
                        error += new_error;
                    }
                }
                continue;
            }
            // Chunk start
            else if self.file_iterator.match_string("[")
            {
                if current_indentation != 0 {
                    chunk.content.push('[');
                }
                current_indentation += 1;
                found_body = true;
            }
            // Chunk end
            else if self.file_iterator.match_string("]")
            {
                if current_indentation != 1 {
                    chunk.content.push(']');
                }
                current_indentation -= 1;
            }
            // Failed to find chunk
            else {
                if !found_body && !Self::is_void(self.file_iterator.current())
                {
                    error.push(self.file_iterator.current_line() as isize, -1, format!("expected '[' but found {}", self.file_iterator.get_next_line()).as_str(), file_path.to_str().unwrap());
                    break;
                }

                if !is_init
                {
                    is_init = true;
                    chunk.line_start = self.file_iterator.current_line() as u32;
                }

                chunk.content.push(self.file_iterator.current());
            }

            self.file_iterator += 1;
        }
        // No end
        if current_indentation != 0 {
            error.push(self.file_iterator.current_line() as isize, -1, format!("chunk doesn't end correctly : {}", chunk.content).as_str(), file_path.to_str().unwrap());
        }
        // No body
        if !found_body {
            error.push(self.file_iterator.current_line() as isize, -1, "failed to find chunk body", file_path.to_str().unwrap());
        }

        if error.empty() {
            return Ok(chunk);
        }
        Err(error)
    }

    fn property_trim_func(chr: char) -> bool {
        chr == ';' || chr == '=' || chr == '\t' || chr == '\n' || chr == '\r' || chr == ' ' || chr == '\'' || chr == '"' || chr == ',' || chr == '(' || chr == ')'
    }

    fn trim_string(string: &String) -> String {
        let mut result = String::new();
        let mut begin: i32 = -1;
        let mut end: i32 = -1;
        let mut chars = string.chars();
        for i in 0..(string.len() - 1) {
            if begin < 0 && !Self::property_trim_func(chars.nth(i).unwrap()) {
                begin = i as i32;
                break;
            }
        }
        for i in (string.len() - 1)..0 {
            if end < 0 && !Self::property_trim_func(chars.nth(i).unwrap()) {
                end = i as i32;
                break;
            }
        }

        if begin >= 0 && end >= 0 {
            for i in begin..end {
                result.push(chars.nth(i as usize).unwrap());
            }
        }

        result
    }

    fn is_void(chr: char) -> bool {
        return chr == ' ' || chr == '\t' || chr == '\r' || chr == '\n';
    }

    fn split_string(string: &String, separators: &Vec<char>) -> Vec<String> {
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

    fn parse_head(header: &String) -> Result<Vec<(String, String)>, ShaderErrorResult>
    {
        let mut fields = Vec::new();
        let mut errors = ShaderErrorResult::default();
        let head_lines = Self::split_string(header, &vec![';']);
        for i in 0..(head_lines.len() - 1)
        {
            let prop_field = Self::split_string(&head_lines[i], &vec!['=']);
            if prop_field.len() != 2
            {
                errors.push(i as isize, -1, &format!("syntax error").to_string(), "");
                continue;
            }

            fields.push((Self::trim_string(&prop_field[0]), Self::trim_string(&prop_field[1])));
        }
        if !errors.empty() {
            return Err(errors);
        }
        Ok(fields)
    }

    fn parse_chunk_head(header: &String) -> Vec<String>
    {
        let mut passes = Vec::new();
        for field in Self::split_string(header, &vec![',']) {
            passes.push(Self::trim_string(&field));
        }
        return passes;
    }

    fn get_property(&self, field: &str) -> &str
    {
        match self.internal_properties.get(field) {
            None => { "" }
            Some(property) => { property.as_str() }
        }
    }

    pub fn new(file_path: &Path) -> Result<Self, ShaderErrorResult> {
        let mut errors = ShaderErrorResult::default();

        let mut shader_code = match fs::read_to_string(file_path) {
            Ok(file_data) => { file_data }
            Err(error) => {
                errors.push(-1, -1, &format!("failed to open file : {}", error), file_path.to_str().unwrap());
                return Err(errors);
            }
        };

        let mut parser = Self {
            file_iterator: FileIterator::new(shader_code),
            includer: Box::new(GlslangIncluder::new()),
            internal_properties: HashMap::new(),
            properties: ShaderProperties::default(),
            globals: Vec::new(),
            default_values: HashMap::new(),
            passes: HashMap::new(),
        };

        match parser.parse_shader(file_path) {
            Ok(_) => {}
            Err(error) => { errors += error }
        };

        if !errors.empty() {
            Err(errors)
        } else {
            Ok(parser)
        }
    }

    fn parse_shader(&mut self, file_path: &Path) -> Result<(), ShaderErrorResult>
    {
        let mut errors = ShaderErrorResult::default();

        while self.file_iterator.valid()
        {
            self.skip_comment();

            if self.file_iterator.match_string("#pragma")
            {
                let pragma_directive = self.file_iterator.get_next_line();
                let fields = Self::split_string(&pragma_directive, &vec![' ', '\t']);
                self.internal_properties.insert(fields[0].clone(), if fields.len() < 2 { String::default() } else { fields[1].clone() });
            }

            if self.file_iterator.match_string("head")
            {
                match self.get_next_chunk(file_path) {
                    Ok(chunk) => {
                        for field in Self::parse_head(&chunk.content) {
                            // self.default_values.insert(field);
                        }
                    }
                    Err(error) => {
                        errors += error;
                        errors.push(self.file_iterator.current_line() as isize, 0, "failed to read 'head' chunk", file_path.to_str().unwrap());
                    }
                };
            }

            if self.file_iterator.match_string("global")
            {
                let _global_args = self.get_next_definition();
                match self.get_next_chunk(file_path) {
                    Ok(chunk) => {
                        let mut chunk_data = chunk.clone();
                        if chunk_data.file.is_empty() {
                            chunk_data.file = file_path.to_str().unwrap().to_string();
                        }
                        self.globals.push(chunk_data);
                    }
                    Err(error) => {
                        errors += error;
                        errors.push(self.file_iterator.current_line() as isize, 0, "failed to read 'global' chunk", file_path.to_str().unwrap());
                    }
                };
            }

            if self.file_iterator.match_string("vertex")
            {
                let vertex_args = self.get_next_definition();
                match self.get_next_chunk(&file_path) {
                    Ok(vertex_chunk) => {
                        let mut chunk_data = vertex_chunk.clone();
                        if chunk_data.file.is_empty() {
                            chunk_data.file = file_path.to_str().unwrap().to_string();
                        }
                        for pass in Self::parse_chunk_head(&vertex_args) {
                            match self.passes.get(&pass) {
                                None => {}
                                Some(elem) => { /*elem.vertex_chunks.push(chunk_data); */ }
                            }
                        }
                    }
                    Err(error) => {
                        errors += error;
                        errors.push(self.file_iterator.current_line() as isize, 0, "failed to read 'vertex' chunk", file_path.to_str().unwrap());
                    }
                };
            }

            if self.file_iterator.match_string("fragment")
            {
                let fragment_args = self.get_next_definition();
                match self.get_next_chunk(&file_path) {
                    Ok(fragment_chunk) => {
                        let mut chunk_data = fragment_chunk.clone();
                        if chunk_data.file.is_empty() {
                            chunk_data.file = file_path.to_str().unwrap().to_string();
                        }
                        for pass in Self::parse_chunk_head(&fragment_args) {
                            // self.passes[pass].vertex_chunks.push(chunk_data);
                        }
                    }
                    Err(error) => {
                        errors += error;
                        errors.push(self.file_iterator.current_line() as isize, 0, "failed to read 'fragment' chunk", file_path.to_str().unwrap());
                    }
                }
            }

            self.file_iterator += 1;
        }
        /*
                for pass in self.passes
                {
                    for global in self.globals
                    {
                       // pass.second.fragment_chunks.insert(pass.second.fragment_chunks.begin(), &global);
                        // pass.second.vertex_chunks.insert(pass.second.vertex_chunks.begin(), &global);
                    }
                }
         */

        self.properties = ShaderProperties {
            shader_version: self.get_property("shader_version").to_string(),
            shader_language: match self.get_property("shader_language") {
                "GLSL" => { ShaderLanguage::GLSL }
                "HLSL" => { ShaderLanguage::HLSL }
                _ => { ShaderLanguage::default() }
            },
            culling: match self.get_property("cull") {
                "FRONT" => { Culling::Front }
                "BACK" => { Culling::Back }
                "BOTH" => { Culling::Both }
                "NONE" => { Culling::None }
                _ => { Culling::default() }
            },
            front_face: match self.get_property("front_face") {
                "CLOCKWISE" => { FrontFace::Clockwise }
                "COUNTER_CLOCKWISE" => { FrontFace::CounterClockwise }
                _ => { FrontFace::default() }
            },
            topology: match self.get_property("topology") {
                "TRIANGLES" => { Topology::Triangles }
                "POINTS" => { Topology::Points }
                "LINES" => { Topology::Lines }
                _ => { Topology::default() }
            },
            polygon_mode: match self.get_property("polygon") {
                "FILL" => { PolygonMode::Fill }
                "POINT" => { PolygonMode::Point }
                "LINE" => { PolygonMode::Line }
                _ => { PolygonMode::default() }
            },
            alpha_mode: match self.get_property("alpha_mode") {
                "OPAQUE" => { AlphaMode::Opaque }
                "TRANSLUCENT" => { AlphaMode::Translucent }
                "ADDITIVE" => { AlphaMode::Additive }
                _ => { AlphaMode::default() }
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
    
    pub fn get_vertex_code(&self, pass: &String) -> Result<Vec<ShaderChunk>, ShaderErrorResult> {
        let mut result = Vec::new();
        let mut errors = ShaderErrorResult::default();
        
        match self.passes.get(pass) {
            None => {
                errors.push(-1, -1, format!("there is no pass named {pass}").as_str(), "");
                return Err(errors);
            }
            Some(element) => {


                for item in self.globals {
                    result.push(item);
                }

                for item in element.vertex_chunks {
                    result.push(item);
                }                
            }
        }        
        
        Ok(result)
    }

    pub fn get_fragment_code(&self, pass: &String) -> Result<Vec<ShaderChunk>, ShaderErrorResult> {
        let mut result = Vec::new();
        let mut errors = ShaderErrorResult::default();

        match self.passes.get(pass) {
            None => {
                errors.push(-1, -1, format!("there is no pass named {pass}").as_str(), "");
                return Err(errors);
            }
            Some(element) => {
                for item in self.globals {
                    result.push(item);
                }

                for item in element.fragment_chunks {
                    result.push(item);
                }
            }
        }

        Ok(result)
    }
    
    pub fn get_available_passes(&self) -> Vec<String> {
        let mut result = Vec::new();
        for (key, value) in self.passes {
            result.push(key.clone());
        }
        result
    }
}

