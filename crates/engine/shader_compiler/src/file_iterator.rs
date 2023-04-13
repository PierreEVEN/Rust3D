use std::ops::{Add, AddAssign, Sub};

pub struct FileIterator {
    line_count: i64,
    ptr: i64,
    shader: String,
    end: char,
}

impl Add<i64> for FileIterator {
    type Output = char;

    fn add(self, rhs: i64) -> Self::Output {
        match self.shader.chars().nth((self.ptr + rhs) as usize) {
            None => { self.end }
            Some(value) => { value }
        }
    }
}

impl Add<i64> for &FileIterator {
    type Output = char;

    fn add(self, rhs: i64) -> Self::Output {
        match self.shader.chars().nth((self.ptr + rhs) as usize) {
            None => { self.end }
            Some(value) => { value }
        }
    }
}

impl Sub<i64> for FileIterator {
    type Output = char;

    fn sub(self, rhs: i64) -> Self::Output {
        self.add(-rhs)
    }
}

impl AddAssign<i64> for FileIterator {
    fn add_assign(&mut self, rhs: i64) {
        for _ in 0..rhs {
            match self.shader.chars().nth((self.ptr) as usize) {
                None => {}
                Some(value) => {
                    if value == '\n' {
                        self.line_count += 1;
                    }
                }
            }
            self.ptr += 1;
        }
    }
}

impl PartialEq<char> for FileIterator {
    fn eq(&self, other: &char) -> bool {
        self.current() == *other
    }
}

impl FileIterator {
    pub fn new(shader_code: &str) -> Self {
        Self {
            line_count: 1,
            ptr: 0,
            shader: shader_code.to_string(),
            end: '\0',
        }
    }

    pub fn current_line(&self) -> i64 {
        self.line_count
    }

    pub fn valid(&self) -> bool {
        self.ptr >= 0 && self.ptr < self.shader.len() as i64
    }

    pub fn match_string(&mut self, str: &str) -> bool {
        if self.match_string_in_place(str) {
            self.add_assign(str.len() as i64);
            true
        } else {
            false
        }
    }

    pub fn match_string_in_place(&mut self, str: &str) -> bool {
        let mut offset = 0;
        while self.valid() && offset < str.len() {
            let current_char = str.chars().nth(offset).expect("failed to read char");
            let chr = self.next(offset as i64);
            if chr != current_char {
                return false;
            }
            offset += 1;
        }
        offset == str.len()
    }

    pub fn get_next_line(&mut self) -> String {
        let mut str = String::new();
        while self.valid() && self.current() != '\n' {
            str.push(self.current());
            self.add_assign(1);
        }
        str
    }

    pub fn current(&self) -> char {
        match self.shader.chars().nth(self.ptr as usize) {
            None => { self.end }
            Some(value) => { value }
        }
    }

    pub fn next(&self, offset: i64) -> char {
        match self.shader.chars().nth((self.ptr + offset) as usize) {
            None => { self.end }
            Some(value) => { value }
        }
    }
}