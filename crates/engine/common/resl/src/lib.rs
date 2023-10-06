use std::fs;
use std::path::PathBuf;
use shader_base::pass_id::PassID;
use shader_base::{ShaderInterface, ShaderStage};

mod ast;
mod list_of;
pub mod resl;


#[derive(Default)]
pub struct ReslShaderInterface {
    _parser: Option<resl::Parser>
}

impl From<PathBuf> for ReslShaderInterface {
    fn from(file_path: PathBuf) -> Self {
        let resl_code = match fs::read_to_string(file_path.clone()) {
            Ok(code) => { code }
            Err(error) => {
                let absolute_path = if file_path.is_absolute() {
                    file_path.to_path_buf()
                } else {
                    std::env::current_dir().unwrap().join(file_path)
                };
                logger::error!("Failed to open file {} : {}", absolute_path.to_str().unwrap(), error);
                return Self::default();
            }
        };

        let mut parser = resl::Parser::default();
        match parser.parse(resl_code, file_path.clone()) {
            Ok(_) => {}
            Err(err) => {
                match err.token {
                    None => {
                        logger::error!("{}\n  --> {}", err.message, file_path.to_str().unwrap());
                    }
                    Some(token) => {
                        let (line, column) = parser.get_error_location(token);
                        logger::error!("{}\n  --> {}:{}:{}", err.message, file_path.to_str().unwrap(), line, column);
                    }
                }
                return Self::default()
            }
        };
        Self {
            _parser: Some(parser)
        }
    }
}

impl ShaderInterface for ReslShaderInterface {
    fn get_spirv_for(&self, _render_pass: &PassID, _stage: ShaderStage) -> Vec<u8> {
        todo!()
    }
}

#[test]
fn parse_resl() {
    let crate_path = "crates/engine/common/resl/";
    let file_path = "src/shader.resl";
    let absolute_path = std::path::PathBuf::from(crate_path.to_string() + file_path);

    let code = std::fs::read_to_string(file_path).unwrap();

    let mut builder = resl::Parser::default();
    match builder.parse(code, absolute_path.clone()) {
        Ok(_) => {}
        Err(err) => {
            match err.token {
                None => {
                    panic!("{}\n  --> {}", err.message, absolute_path.to_str().unwrap());
                }
                Some(token) => {
                    let (line, column) = builder.get_error_location(token);
                    panic!("{}\n  --> {}:{}:{}", err.message, absolute_path.to_str().unwrap(), line, column);
                }
            }
        }
    };

    println!("{:?}", builder.hlsl);
}