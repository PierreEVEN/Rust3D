use std::collections::{HashMap};
use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::sync::{Arc, RwLock};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, RECT, WPARAM, HMODULE};
use windows::Win32::Graphics::Gdi::{BLACK_BRUSH, EnumDisplayMonitors, GetMonitorInfoW, GetStockObject, HBRUSH, HDC, HMONITOR, MONITORINFO};
use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, DefWindowProcW, DispatchMessageW, GET_CLASS_LONG_INDEX, GetClassLongPtrW, HMENU, IDC_ARROW, LoadCursorW, PeekMessageW, PM_REMOVE, RegisterClassExW, SetClassLongPtrW, TranslateMessage, UnregisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSEXW};
use windows::Win32::UI::WindowsAndMessaging::*;

use maths::rect2d::Rect2D;
use plateform::{Monitor, Platform, WindowCreationError};
use plateform::input_system::InputManager;
use plateform::window::{PlatformEvent, Window, WindowCreateInfos};

use crate::utils::{check_win32_error, utf8_to_utf16};
use crate::win32_inputs::win32_input;
use crate::window::WindowWin32;

mod window;
mod utils;
mod win32_inputs;

const WIN_CLASS_NAME: &str = "r3d_window";

#[derive(Eq, PartialEq)]
struct HashableHWND(HWND);

impl Hash for HashableHWND {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_isize(self.0.0);
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

impl PlatformWin32 {
    pub fn new() -> Arc<PlatformWin32> {
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

        let platform = Arc::new(PlatformWin32 {
            windows: Default::default(),
            monitors: Default::default(),
            input_manager: InputManager::new(),
        });

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
                    (platform.as_ref() as *const PlatformWin32) as isize,
                );

                match check_win32_error() {
                    Ok(_) => (),
                    Err(_message) => logger::fatal!("failed to send platform pointer to wndclass : {_message}"),
                }
            }
        }

        // Collect monitors
        platform.collect_monitors();

        platform
    }


    fn send_window_message(&self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        let window_map = self.windows.read();
        if let Some(window) = window_map.unwrap().get(&hwnd.into()) {
            win32_input(msg, wparam, lparam, &self.input_manager);

            match msg {
                WM_CLOSE => {
                    window.trigger_event(&PlatformEvent::WindowClosed)
                }
                WM_SIZE => {
                    window.trigger_event(&PlatformEvent::WindowResized(win32_loword!(lparam.0), win32_hiword!(lparam.0)));
                }
                _ => {}
            }
        }
    }
}

impl Drop for PlatformWin32 {
    fn drop(&mut self) {
        unsafe {
            UnregisterClassW(
                PCWSTR(utf8_to_utf16(WIN_CLASS_NAME).as_ptr()),
                HMODULE::default(),
            );
            timeEndPeriod(1);
        }
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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
    fn create_window(&self, create_infos: WindowCreateInfos) -> Result<Arc<dyn Window>, WindowCreationError> {
        let window = WindowWin32::new(create_infos);
        let hwnd = window.hwnd.into();
        self.windows.write().unwrap().insert(hwnd, window.clone());
        Ok(window)
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

unsafe extern "system" fn enum_display_monitors_callback(monitor: HMONITOR, _: HDC, _: *mut RECT, userdata: LPARAM) -> BOOL {
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
        Err(error) => logger::fatal!("failed to get DPI for monitor {}", error)
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
