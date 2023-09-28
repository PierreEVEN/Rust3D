mod ast;

use std::fs;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub languagev2); // synthesized by LALRPOP

fn get_error_location(code: &str, token: usize) -> (usize, usize) {
    let mut line = 0;
    let mut token_to_last_line = 0;
    let mut elapsed_tokens = 0;
    for chr in code.chars() {
        if chr == '\n' {
            line += 1;
            token_to_last_line = elapsed_tokens;
        }
        elapsed_tokens += 1;
        if elapsed_tokens >= token {
            break;
        }
    }
    (line, token - token_to_last_line)
}

#[test]
fn calculator1() {

    let crate_path = "crates\\engine\\common\\resl\\";
    let file_path = "shader.resl";

    let code = fs::read_to_string(file_path).unwrap();

    let parsed = match languagev2::InstructionListParser::new().parse(code.as_str()) {
        Ok(res) => res,
        Err(e) => match e {
            lalrpop_util::ParseError::UnrecognizedToken {token, expected} => {
                let (a, b, c) = token;
                // Comment lire le contenu de UnrecognizedToken ?
                let (line, column) = get_error_location(code.as_str(), a);

                panic!("Unrecognized token {:?} : expected {:?}\n  --> {}{}:{}:{}", b, expected, crate_path, file_path, line, column)
            },
            lalrpop_util::ParseError::InvalidToken {location} => {
                let (line, column) = get_error_location(code.as_str(), location);
                panic!("Invalid token at ({}:{}) : '{:?}'", file_path, line, code.chars().nth(location).unwrap())
            },
            _ => {
                panic!("Compilation failed:\n{:?}", e)
            }
        }
    };

    println!("${:?}", parsed);
}