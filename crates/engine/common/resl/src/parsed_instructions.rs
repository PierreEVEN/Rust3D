use std::ops::{AddAssign};

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct ParsedInstructions {
    instructions: Vec<(usize, String)>,
}

impl ParsedInstructions {
    pub fn get_text(&self) -> String {
        let mut text = String::new();
        for (_, instr) in &self.instructions {
            text += instr.as_str()
        }
        text
    }
    #[allow(dead_code)]
    pub fn get_token_for(&self, line: usize, column: usize) -> Option<usize> {
        let mut cur_line = 1;
        let mut good_line = false;
        let mut column_offset = 0;
        for (token, instr) in &self.instructions {
            if instr == "\n" {
                cur_line += 1;
                if good_line { return None; }
                if line == cur_line {
                    good_line = true;
                }
            } else if good_line
            {
                column_offset += instr.len();
                if column_offset > column {
                    return Some(*token)
                }
            }
        }
        None
    }
}

impl AddAssign<(usize, String)> for ParsedInstructions {
    fn add_assign(&mut self, rhs: (usize, String)) {
        self.instructions.push(rhs)
    }
}

impl AddAssign<(&usize, &String)> for ParsedInstructions {
    fn add_assign(&mut self, (left, right): (&usize, &String)) {
        self.instructions.push((*left, right.clone()))
    }
}

impl AddAssign<(usize, &str)> for ParsedInstructions {
    fn add_assign(&mut self, (token, string): (usize, &str)) {
        self.instructions.push((token, string.to_string()))
    }
}

impl AddAssign<(&usize, &str)> for ParsedInstructions {
    fn add_assign(&mut self, (token, string): (&usize, &str)) {
        self.instructions.push((*token, string.to_string()))
    }
}

impl AddAssign<ParsedInstructions> for ParsedInstructions {
    fn add_assign(&mut self, rhs: ParsedInstructions) {
        let mut rhs = rhs;
        self.instructions.append(&mut rhs.instructions)
    }
}

impl AddAssign<&mut ParsedInstructions> for ParsedInstructions {
    fn add_assign(&mut self, rhs: &mut ParsedInstructions) {
        self.instructions.append(&mut rhs.instructions)
    }
}