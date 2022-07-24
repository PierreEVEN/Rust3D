use std::ptr::null;
use std::sync::{Arc};
use raw_window_handle::RawWindowHandle;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRectEx, CreateWindowExW, HMENU, LWA_ALPHA, SetLayeredWindowAttributes, ShowWindow, SW_MAXIMIZE, SW_SHOW, WINDOW_STYLE, WS_CAPTION, WS_EX_LAYERED, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE};
use maths::rect2d::{RectI32};
use plateform::window::{Window, WindowCreateInfos, WindowFlagBits, WindowFlags};
use crate::{utf8_to_utf16, WIN_CLASS_NAME};
use crate::utils::check_win32_error;

pub struct WindowWin32 {
    pub hwnd: HWND,
    flags: WindowFlags,
    geometry: RectI32,
}

impl WindowWin32 {
    pub fn new(create_infos: WindowCreateInfos) -> Arc<WindowWin32> {
        let ex_style = WS_EX_LAYERED;
        let mut style = WINDOW_STYLE::default();
        
        unsafe {
            if create_infos.window_flags.contains(WindowFlagBits::Borderless) {
                style |= WS_VISIBLE | WS_POPUP;
            } else {
                style |= WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
            }

            if create_infos.window_flags.contains(WindowFlagBits::Resizable) {
                style |= WS_THICKFRAME | WS_MAXIMIZEBOX;
            }

            // Rect must be adjusted since Win32 api include window decoration in the width/height
            let mut initial_rect = RECT {
                left: 0,
                top: 0,
                right: create_infos.geometry.width() as i32,
                bottom: create_infos.geometry.height() as i32,
            };

            AdjustWindowRectEx(&mut initial_rect, style, false, ex_style);
            let hwnd = CreateWindowExW(
                ex_style,
                PCWSTR(utf8_to_utf16(WIN_CLASS_NAME).as_ptr()),
                PCWSTR(utf8_to_utf16(create_infos.name.as_str()).as_ptr()),
                style,
                create_infos.geometry.min_x() as i32 + initial_rect.left,
                create_infos.geometry.min_y() as i32 + initial_rect.top,
                initial_rect.right - initial_rect.left,
                initial_rect.bottom - initial_rect.top,
                HWND::default(),
                HMENU::default(),
                HINSTANCE::default(),
                null(),
            );

            match check_win32_error() {
                Err(_message) => { panic!("failed to create window : {_message}"); }
                Ok(_) => {}
            }

            let window = Arc::new(WindowWin32 {
                hwnd,
                geometry: create_infos.geometry,
                flags: create_infos.window_flags,
            });
            window.set_background_alpha(create_infos.background_alpha);
            
            return window;
        }
    }
}

impl Window for WindowWin32 {
    fn set_geometry(&mut self, _geometry: RectI32) {
        self.geometry = _geometry.clone()
    }

    fn set_title(&self, _title: &str) {
        todo!()
    }

    fn show(&self) {
        unsafe {
            ShowWindow(
                self.hwnd,
                if self.flags.contains(WindowFlagBits::Maximized) {
                    SW_MAXIMIZE
                } else {
                    SW_SHOW
                },
            );
        }
    }

    fn set_background_alpha(&self, alpha: u8) {
        unsafe {
            SetLayeredWindowAttributes(self.hwnd, 0, alpha, LWA_ALPHA);
        }
    }

    fn get_handle(&self) -> RawWindowHandle {
        todo!()
    }

    fn get_geometry(&self) -> RectI32 {
        todo!()
    }
}