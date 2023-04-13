
use crate::types::ShaderErrorResult;

pub trait Includer {
    fn include_local(&self, file: &str, virtual_path: &str) -> Result<(String, String), ShaderErrorResult>;
    fn include_system(&self, file: &str, virtual_path: &str) -> Result<(String, String), ShaderErrorResult>;
    fn release_include(&self, file: &str, virtual_path: &str);
    fn add_include_path(&self, virtual_path: &str);
}