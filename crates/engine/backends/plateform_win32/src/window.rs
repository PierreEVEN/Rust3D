use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HMODULE, HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRectEx, CreateWindowExW, GetSystemMetrics, HMENU, LWA_ALPHA, SetLayeredWindowAttributes, SetWindowTextW, ShowWindow, SM_CXSCREEN, SM_CYCAPTION,  SM_CYSCREEN, SW_MAXIMIZE, SW_SHOW, WINDOW_STYLE, WS_CAPTION, WS_EX_LAYERED, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE};

use logger::{info};
use maths::rect2d::{Rect2D, RectI32};
use plateform::window::{
    PlatformEvent, Window, WindowCreateInfos, WindowEventDelegate, WindowFlagBits, WindowFlags,
};
use plateform::window::WindowFlagBits::Maximized;

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

        let mut window_flags = create_infos.window_flags;
        
        unsafe {
            if window_flags
                .contains(WindowFlagBits::Borderless)
            {
                style |= WS_VISIBLE | WS_POPUP;
            } else {
                style |= WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
            }

            if window_flags
                .contains(WindowFlagBits::Resizable)
            {
                style |= WS_THICKFRAME | WS_MAXIMIZEBOX;
            }

            let geometry = match create_infos.geometry {
                None => {
                    //window_flags.insert(Maximized);
                    let width = GetSystemMetrics(SM_CXSCREEN);
                    let height = GetSystemMetrics(SM_CYSCREEN);
                    Rect2D::new(width / 4, GetSystemMetrics(SM_CYCAPTION) + height / 4, width - width / 4, height - height / 4)
                }
                Some(geometry) => { geometry }
            };

            // Rect must be adjusted since Win32 api include window decoration in the width/height
            let mut initial_rect = RECT {
                left: 0,
                top: 0,
                right: geometry.width(),
                bottom: geometry.height(),
            };
            AdjustWindowRectEx(&mut initial_rect, style, false, ex_style);
            
            let hwnd = CreateWindowExW(
                ex_style,
                PCWSTR(utf8_to_utf16(WIN_CLASS_NAME).as_ptr()),
                PCWSTR(utf8_to_utf16(create_infos.name.as_str()).as_ptr()),
                style,
                geometry.min_x() + initial_rect.left,
                geometry.min_y() + initial_rect.top,
                initial_rect.right - initial_rect.left,
                initial_rect.bottom - initial_rect.top,
                HWND::default(),
                HMENU::default(),
                HMODULE::default(),
                None,
            );

            if let Err(_message) = check_win32_error() {
                logger::fatal!("failed to create window : {_message}");
            }

            let window = WindowWin32 {
                hwnd,
                geometry: RwLock::new(geometry),
                background_alpha: RwLock::new(create_infos.background_alpha),
                flags: window_flags,
                title: RwLock::new(create_infos.name.to_string()),
                event_map: RwLock::default(),
            };

            window.set_background_alpha(create_infos.background_alpha);

            Arc::new(window)
        }
    }

    pub fn trigger_event(&self, event_type: &PlatformEvent) {
        match self.event_map.write().unwrap().get_mut(event_type) {
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
        (*self.geometry.write().unwrap()) = _geometry
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
            SetLayeredWindowAttributes(self.hwnd, COLORREF::default(), alpha, LWA_ALPHA);
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
        self.event_map
            .write()
            .unwrap()
            .insert(event_type, vec![delegate]);
    }
}

impl Drop for WindowWin32 {
    fn drop(&mut self) {
        info!("Destroy window '{}'", *self.title.read().unwrap());
    }
}
