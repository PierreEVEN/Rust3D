mod ast;
mod list_of;
pub mod hlsl_builder;

#[test]
fn parse_resl() {
    let crate_path = "crates/engine/common/resl/";
    let file_path = "src/shader.resl";
    let absolute_path = std::path::PathBuf::from(crate_path.to_string() + file_path);

    let code = std::fs::read_to_string(file_path).unwrap();

    let mut builder = hlsl_builder::ReslParser::default();
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