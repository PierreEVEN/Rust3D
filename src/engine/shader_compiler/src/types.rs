﻿use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::{fmt, ops};

use gfx::types::PixelFormat;

use crate::ShaderChunk;

#[derive(Clone)]
pub struct ShaderError {
    text: String,
    file_path: String,
    line: isize,
    column: isize,
}

#[derive(Clone, Default)]
pub struct ShaderErrorResult {
    error_list: Vec<ShaderError>,
}

impl ShaderErrorResult {
    pub fn push(&mut self, line: isize, column: isize, message: &str, file_path: &str) {
        self.error_list.push(ShaderError {
            text: message.to_string(),
            file_path: file_path.to_string(),
            line,
            column,
        })
    }
    pub fn empty(&self) -> bool {
        self.error_list.len() == 0
    }
}

impl ToString for ShaderErrorResult {
    fn to_string(&self) -> String {
        let mut result = String::from("failed to parse shader :\n");
        let mut index = 1;
        for error in &self.error_list {
            if error.line >= 0 && error.column >= 0 {
                result += format!("\t{}: [{}:{}:{}]\n\t\t{}\n", index, error.file_path, error.line, error.column, error.text).as_str();                
            }
            else if error.line >= 0 {
                result += format!("\t{}: [{}:{}]\n\t\t{}\n", index, error.file_path, error.line, error.text).as_str();
            }
            else {
                result += format!("\t{}: [{}]\n\t\t{}\n", index, error.file_path, error.text).as_str();                
            }
            
            index += 1;
        }
        result
    }
}

impl ops::AddAssign<ShaderErrorResult> for ShaderErrorResult {
    fn add_assign(&mut self, rhs: ShaderErrorResult) {
        for error in rhs.error_list {
            self.error_list.push(error.clone());
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct ShaderBlock
{
    pub name: String,
    pub raw_text: String,
}

#[derive(Clone, Debug)]
pub enum ShaderLanguage
{
    HLSL,
    GLSL,
}

impl Default for ShaderLanguage {
    fn default() -> Self {
        ShaderLanguage::HLSL
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ShaderStage
{
    Vertex,
    Fragment,
}

impl Display for ShaderStage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ShaderStage::Vertex => write!(f, "Vertex"),
            ShaderStage::Fragment => write!(f, "Fragment"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Culling
{
    None,
    Front,
    Back,
    Both,
}

impl Default for Culling {
    fn default() -> Self {
        Culling::Back
    }
}

#[derive(Clone, Debug)]
pub enum FrontFace
{
    Clockwise,
    CounterClockwise,
}

impl Default for FrontFace {
    fn default() -> Self {
        FrontFace::CounterClockwise
    }
}

#[derive(Clone, Debug)]
pub enum Topology
{
    Points,
    Lines,
    Triangles,
}

impl Default for Topology {
    fn default() -> Self {
        Topology::Triangles
    }
}

#[derive(Clone, Debug)]
pub enum PolygonMode
{
    Point,
    Line,
    Fill,
}

impl Default for PolygonMode {
    fn default() -> Self {
        PolygonMode::Fill
    }
}

#[derive(Clone, Debug)]
pub enum AlphaMode
{
    Opaque,
    Translucent,
    Additive,
}

impl Default for AlphaMode {
    fn default() -> Self {
        AlphaMode::Opaque
    }
}

struct TypeInfo
{
    type_name: String,
    type_id: c_void,
    type_size: usize,
    format: PixelFormat,
}

struct Property
{
    name: String,
    type_info: TypeInfo,
    offset: u32,
    location: i32,
}

pub struct ShaderProperties
{
    pub shader_version: String,
    pub shader_language: ShaderLanguage,
    pub culling: Culling,
    pub front_face: FrontFace,
    pub topology: Topology,
    pub polygon_mode: PolygonMode,
    pub alpha_mode: AlphaMode,
    pub depth_test: bool,
    pub line_width: f32,
}

impl Default for ShaderProperties {
    fn default() -> Self {
        Self {
            shader_version: "1.0".to_string(),
            shader_language: Default::default(),
            culling: Default::default(),
            front_face: Default::default(),
            topology: Default::default(),
            polygon_mode: Default::default(),
            alpha_mode: Default::default(),
            depth_test: true,
            line_width: 1.0,
        }
    }
}

pub struct InterstageData
{
    pub stage_outputs: HashMap<String, u32>,
    pub binding_index: i32,
}