use windows::Win32::Foundation::{GetLastError, NO_ERROR};

pub fn utf8_to_utf16(str: &str) -> Vec<u16> {
    str.encode_utf16().chain(Some(0)).collect()
}

pub fn loword(word: isize) -> i32 {
    (word & 0xffff) as i32
}

pub fn hiword(word: isize) -> i32 {
    ((word >> 16) & 0xffff) as i32
}

pub fn check_win32_error() -> Result<(), String> {
    unsafe {
        let last_error = GetLastError();
        if last_error != NO_ERROR {
            return Err(format!("Win32 API [{}]", last_error.0));
        }
    }
    Ok(())
}

#[macro_export]
macro_rules! win32_loword {
    ($word:expr) => {
        ($word & 0xffff) as u32
    };
}

#[macro_export]
macro_rules! win32_hiword {
    ($word:expr) => {
        ($word >> 16 & 0xffff) as u32
    };
}
