
use crate::types::ShaderErrorResult;

pub trait Includer {
    fn include_local(&self, file: &String, virtual_path: &String) -> Result<(String, String), ShaderErrorResult>;
    fn include_system(&self, file: &String, virtual_path: &String) -> Result<(String, String), ShaderErrorResult>;
    fn release_include(&self, file: &String, virtual_path: &String);
    fn add_include_path(&self, virtual_path: &String);
}