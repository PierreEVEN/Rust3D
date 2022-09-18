use std::collections::HashMap;
use std::ptr::null;
use std::sync::{Arc, RwLock};

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRectEx, CreateWindowExW, HMENU, LWA_ALPHA, SetLayeredWindowAttributes, SetWindowTextW, ShowWindow, SW_MAXIMIZE, SW_SHOW, WINDOW_STYLE, WS_CAPTION, WS_EX_LAYERED, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE};

use maths::rect2d::RectI32;
use plateform::window::{PlatformEvent, Window, WindowCreateInfos, WindowEventDelegate, WindowFlagBits, WindowFlags};

use crate::{utf8_to_utf16, WIN_CLASS_NAME};
use crate::utils::check_win32_error;

pub struct WindowWin32 {
    pub hwnd: HWND,
    flags: WindowFlags,
    geometry: RwLock<RectI32>,
    background_alpha: RwLock<u8>,
    title: RwLock<String>,
    event_map: RwLock<HashMap<PlatformEvent, Vec<WindowEventDelegate>>>,
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

            let window = WindowWin32 {
                hwnd,
                geometry: RwLock::new(create_infos.geometry),
                background_alpha: RwLock::new(create_infos.background_alpha),
                flags: create_infos.window_flags,
                title: RwLock::new(create_infos.name.to_string()),
                event_map: RwLock::default(),
            };

            window.set_background_alpha(create_infos.background_alpha);

            return Arc::new(window);
        }
    }

    pub fn trigger_event(&self, event_type: &PlatformEvent) {
        match self.event_map.write().unwrap().get_mut(&event_type) {
            None => {}
            Some(events) => {
                for event in events {
                    event(event_type);
                }
            }
        }
    }
}

impl Window for WindowWin32 {
    fn set_geometry(&self, _geometry: RectI32) {
        (*self.geometry.write().unwrap()) = _geometry.clone()
    }

    fn get_geometry(&self) -> RectI32 {
        *self.geometry.read().unwrap()
    }

    fn set_title(&self, _title: &str) {
        unsafe {
            if SetWindowTextW(self.hwnd, PCWSTR(utf8_to_utf16(_title).as_ptr())).as_bool() {
                (*self.title.write().unwrap()) = _title.to_string();
            }
        }
    }

    fn get_title(&self) -> String {
        self.title.read().unwrap().clone()
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
        (*self.background_alpha.write().unwrap()) = alpha;
    }

    fn get_background_alpha(&self) -> u8 {
        *self.background_alpha.read().unwrap()
    }

    fn get_handle(&self) -> RawWindowHandle {
        let mut handle = Win32WindowHandle::empty();
        handle.hwnd = self.hwnd.0 as *mut std::ffi::c_void;
        RawWindowHandle::Win32(handle)
    }

    fn bind_event(&self, event_type: PlatformEvent, delegate: WindowEventDelegate) {
        match self.event_map.write().unwrap().get_mut(&event_type) {
            None => {}
            Some(events) => {
                events.push(delegate);
                return;
            }
        }
        self.event_map.write().unwrap().insert(event_type, vec![delegate]);
    }
}