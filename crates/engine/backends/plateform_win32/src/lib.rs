use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::sync::{Arc, RwLock, Weak};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, HMODULE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, GetStockObject, BLACK_BRUSH, HBRUSH, HDC, HMONITOR,
    MONITORINFO,
};
use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClassLongPtrW, LoadCursorW, PeekMessageW,
    RegisterClassExW, SetClassLongPtrW, TranslateMessage, UnregisterClassW, CS_DBLCLKS, CS_HREDRAW,
    CS_VREDRAW, GET_CLASS_LONG_INDEX, HMENU, IDC_ARROW, PM_REMOVE, WINDOW_EX_STYLE, WINDOW_STYLE,
    WNDCLASSEXW,
};

use maths::rect2d::Rect2D;
use plateform::input_system::InputManager;
use plateform::window::{PlatformEvent, Window, WindowCreateInfos};
use plateform::{Monitor, Platform, WindowCreationError};

use crate::utils::{check_win32_error, utf8_to_utf16};
use crate::win32_inputs::win32_input;
use crate::window::WindowWin32;

mod utils;
mod win32_inputs;
mod window;

const WIN_CLASS_NAME: &str = "r3d_window";

#[derive(Eq, PartialEq)]
struct HashableHWND(HWND);

impl Hash for HashableHWND {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_isize(self.0 .0);
    }
}

impl From<HWND> for HashableHWND {
    fn from(hwnd: HWND) -> Self {
        Self(hwnd)
    }
}

pub struct PlatformWin32 {
    windows: RwLock<HashMap<HashableHWND, Arc<WindowWin32>>>,
    monitors: Vec<Monitor>,
    input_manager: InputManager,
}

impl Default for PlatformWin32 {
    fn default() -> Self {
        unsafe {
            // Ensure time precision is the highest
            timeBeginPeriod(1);

            let win_class = WNDCLASSEXW {
                cbSize: size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
                lpszClassName: PCWSTR(utf8_to_utf16(WIN_CLASS_NAME).as_ptr()),
                hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
                hCursor: LoadCursorW(HMODULE::default(), IDC_ARROW).unwrap(),
                cbClsExtra: size_of::<usize>() as i32,
                lpfnWndProc: Some(wnd_proc),
                ..Default::default()
            };

            assert_ne!(RegisterClassExW(&win_class), 0);
        }

        let platform = PlatformWin32 {
            windows: Default::default(),
            monitors: Default::default(),
            input_manager: InputManager::new(),
        };

        // Set platform pointer into WNDCLASS
        unsafe {
            {
                let window_handle = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    PCWSTR(utf8_to_utf16(WIN_CLASS_NAME).as_ptr()),
                    PCWSTR::null(),
                    WINDOW_STYLE(0),
                    0,
                    0,
                    1,
                    1,
                    HWND::default(),
                    HMENU::default(),
                    HMODULE::default(),
                    None,
                );

                SetClassLongPtrW(
                    window_handle,
                    GET_CLASS_LONG_INDEX(0),
                    (&platform as *const PlatformWin32) as isize,
                );
                DestroyWindow(window_handle).expect("failed to destroy init window handle");

                match check_win32_error() {
                    Ok(_) => (),
                    Err(_message) => {
                        logger::fatal!("failed to send platform pointer to wndclass : {_message}")
                    }
                }
            }
        }

        // Collect monitors
        platform.collect_monitors();
        logger::info!("Created win32 platform backend");
        platform
    }
}

impl PlatformWin32 {
    fn send_window_message(&self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        let window_map = self.windows.read();
        if let Some(window) = window_map.unwrap().get(&hwnd.into()) {
            win32_input(msg, wparam, lparam, &self.input_manager);

            match msg {
                WM_CLOSE => window.trigger_event(&PlatformEvent::WindowClosed),
                WM_SIZE => {
                    window.trigger_event(&PlatformEvent::WindowResized(
                        win32_loword!(lparam.0),
                        win32_hiword!(lparam.0),
                    ));
                }
                _ => {}
            }
        }
    }
}

impl Drop for PlatformWin32 {
    fn drop(&mut self) {
        self.windows.write().unwrap().clear();
        unsafe {
            match UnregisterClassW(
                PCWSTR(utf8_to_utf16(WIN_CLASS_NAME).as_ptr()),
                HMODULE::default(),
            ) {
                Ok(_) => {}
                Err(err) => {logger::error!("Failed to unregister platform class : {}", err)}
            };
            timeEndPeriod(1);
        }
        logger::info!("Destroyed win32 platform backend");
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let platform = {
        let ptr = GetClassLongPtrW(hwnd, GET_CLASS_LONG_INDEX(0));
        if ptr == 0 {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        (ptr as *const PlatformWin32).as_ref().unwrap_unchecked()
    };

    platform.send_window_message(hwnd, msg, wparam, lparam);
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl Platform for PlatformWin32 {
    fn create_window(
        &self,
        create_infos: WindowCreateInfos,
    ) -> Result<Weak<dyn Window>, WindowCreationError> {
        logger::info!("create window '{}'", create_infos.name);
        let window = WindowWin32::new(create_infos);
        let hwnd = window.hwnd.into();
        self.windows.write().unwrap().insert(hwnd, window.clone());
        Ok(Arc::downgrade(&(window as Arc<dyn Window>)))
    }

    fn monitor_count(&self) -> usize {
        self.monitors.len()
    }

    fn get_monitor(&self, index: usize) -> Monitor {
        self.monitors[index]
    }

    fn collect_monitors(&self) {
        unsafe {
            EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(enum_display_monitors_callback),
                LPARAM((&self.monitors as *const _) as isize),
            );
        }

        match check_win32_error() {
            Ok(_) => (),
            Err(_message) => logger::fatal!("failed to get monitor information : {_message}"),
        }
    }

    fn poll_events(&self) {
        unsafe {
            let mut msg = std::mem::zeroed();
            if PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE) != false {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn input_manager(&self) -> &InputManager {
        &self.input_manager
    }
}

unsafe extern "system" fn enum_display_monitors_callback(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    userdata: LPARAM,
) -> BOOL {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        rcMonitor: Default::default(),
        rcWork: Default::default(),
        dwFlags: 0,
    };

    GetMonitorInfoW(monitor, &mut info);

    let mut dpi_x = 0;
    let mut dpi_y = 0;
    match GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) {
        Ok(_) => (),
        Err(error) => logger::fatal!("failed to get DPI for monitor {}", error),
    }

    let monitors = (userdata.0 as *mut Vec<Monitor>)
        .as_mut()
        .unwrap_unchecked();

    monitors.push(Monitor {
        bounds: Rect2D::<i32>::new(
            info.rcMonitor.left,
            info.rcMonitor.top,
            info.rcMonitor.right,
            info.rcMonitor.bottom,
        ),
        work_bounds: Rect2D::<i32>::new(
            info.rcWork.left,
            info.rcWork.top,
            info.rcWork.right,
            info.rcWork.bottom,
        ),
        dpi: dpi_x as f32,
        primary: info.dwFlags & MONITORINFOF_PRIMARY != 0,
    });

    match check_win32_error() {
        Ok(_) => (),
        Err(_message) => logger::fatal!("failed to get monitor information : {_message}"),
    }
    BOOL::from(true)
}
