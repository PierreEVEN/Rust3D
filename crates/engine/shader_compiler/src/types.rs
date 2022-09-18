use std::ops;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ShaderError {
    text: String,
    file_path: String,
    error_id: String,
    line: Option<isize>,
    column: Option<isize>,
}

#[derive(Clone, Default)]
pub struct ShaderErrorResult {
    error_list: Vec<ShaderError>,
}

impl ShaderErrorResult {
    pub fn push(&mut self, line: Option<isize>, column: Option<isize>, error_id: &str, error_message: &str, file_path: &str) {
        self.error_list.push(ShaderError {
            text: error_message.to_string(),
            file_path: file_path.to_string(),
            error_id: error_id.to_string(),
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
        
        
        let mut result = String::from("stack backtrace:\n");
        let mut index = 1;
        for error in &self.error_list {
            let mut text = String::new();
            let multiline = error.text.contains("\n");
            for line in error.text.split("\n") {
                text += "\t\t\t";
                if multiline {
                    text += "| ";
                }
                text += line;
                text += "\n";
            }


            if error.line.is_some() && error.column.is_some() {
                result += format!("\t{}: {}\n\t\tat {}:{}:{}\n{}\n", index, error.error_id, error.file_path, error.line.unwrap(), error.column.unwrap(), text).as_str();
            } else if error.line.is_some() {
                result += format!("\t{}: {}\n\t\tat {}:{}:0\n{}\n", index, error.error_id, error.file_path, error.line.unwrap(), text).as_str();
            } else {
                result += format!("\t{}: {}\n\t\tat {}:0:0\n{}\n", index, error.error_id, error.file_path, text).as_str();
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

pub struct InterstageData
{
    pub stage_outputs: HashMap<String, u32>,
    pub binding_index: i32,
}