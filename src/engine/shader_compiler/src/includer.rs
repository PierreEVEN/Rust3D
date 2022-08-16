use std::path::Path;

use crate::types::ShaderErrorResult;

pub trait Includer {
    fn include_local(&self, file: &String, shader_path: &Path) -> Result<(String, String), ShaderErrorResult>;
    fn include_system(&self, file: &String, shader_path: &Path) -> Result<(String, String), ShaderErrorResult>;
    fn release_include(&self, file: &String, shader_path: &Path);
    fn add_include_path(&self, include_path: &Path);
}