pub mod window;
pub mod utils;

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::ptr::null;
use std::sync::{Arc, Mutex};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{BLACK_BRUSH, EnumDisplayMonitors, GetMonitorInfoW, GetStockObject, HBRUSH, HDC, HMONITOR, MONITORINFO};
use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, DefWindowProcW, DispatchMessageW, GET_CLASS_LONG_INDEX, GetClassLongPtrW, HMENU, IDC_ARROW, LoadCursorW, PeekMessageW, PM_REMOVE, RegisterClassExW, SetClassLongPtrW, TranslateMessage, UnregisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSEXW};
use maths::rect2d::{Rect2D};
use plateform::{Monitor, Platform};
use plateform::window::{Window, WindowCreateInfos, PlatformEvent};
use crate::utils::{check_win32_error, utf8_to_utf16};
use crate::window::WindowWin32;
use windows::Win32::UI::WindowsAndMessaging::*;

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
        Self { 0: hwnd }
    }
}

pub struct PlatformWin32 {
    windows: Mutex<HashMap<HashableHWND, Arc<Mutex<WindowWin32>>>>,
    messages: Mutex<VecDeque<PlatformEvent>>,
    monitors: Vec<Monitor>,
}

impl PlatformWin32 {
    pub fn new() -> Arc<PlatformWin32> {

        unsafe {
            // Ensure time precision is the highest
            timeBeginPeriod(1);

            let mut win_class = WNDCLASSEXW::default();
            win_class.cbSize = size_of::<WNDCLASSEXW>() as u32;
            win_class.style = CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS;
            win_class.lpszClassName = PCWSTR(utf8_to_utf16(WIN_CLASS_NAME).as_ptr());
            win_class.hbrBackground = HBRUSH(GetStockObject(BLACK_BRUSH).0);
            win_class.hCursor = LoadCursorW(HINSTANCE::default(), IDC_ARROW).unwrap();
            win_class.cbClsExtra = size_of::<usize>() as i32;
            win_class.lpfnWndProc = Some(wnd_proc);
            assert_ne!(RegisterClassExW(&win_class), 0);
        }
        
        let platform = Arc::new(PlatformWin32 {
            windows: Default::default(),
            messages: Mutex::new(VecDeque::new()),
            monitors: Default::default(),
        });
        
        // Set platform pointer into WNDCLASS
        unsafe {
            {
                let window_handle = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    PCWSTR(utf8_to_utf16(WIN_CLASS_NAME).as_ptr()),
                    PCWSTR::default(),
                    WINDOW_STYLE(0),
                    0,
                    0,
                    1,
                    1,
                    HWND::default(),
                    HMENU::default(),
                    HINSTANCE::default(),
                    null(),
                );

                SetClassLongPtrW(
                    window_handle,
                    GET_CLASS_LONG_INDEX(0),
                    (platform.as_ref() as *const PlatformWin32) as isize,
                );

                match check_win32_error() {
                    Ok(_) => (),
                    Err(_message) => panic!("failed to send platform pointer to wndclass : {_message}"),
                }
            }
        }
        
        // Collect monitors
        platform.collect_monitors();
        
        return platform;
    }


    fn send_window_message(&self, hwnd: HWND, msg: u32, _wparam: WPARAM, lparam: LPARAM) {
        let window_map = self.windows.lock();
        if let Some(window) = window_map.unwrap().get(&hwnd.into()) {
            let message_queue = self.messages.lock();
            match msg {
                WM_CLOSE => {
                    message_queue.unwrap().push_back(PlatformEvent::WindowClosed(window.clone()));
                }
                WM_SIZE => {
                    message_queue.unwrap().push_back(PlatformEvent::WindowResized(window.clone(), win32_loword!(lparam.0), win32_hiword!(lparam.0)));
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
                HINSTANCE::default(),
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
    fn create_window(&self, create_infos: WindowCreateInfos) -> Result<Arc<Mutex<dyn Window>>, ()> {
        let window = WindowWin32::new(create_infos.clone());
        let hwnd = window.lock().unwrap().hwnd.into(); 
        self.windows.lock().unwrap().insert(hwnd, window.clone());
        return Ok(window);
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
                null(),
                Some(enum_display_monitors_callback),
                LPARAM((&self.monitors as *const _) as isize),
            );
        }

        match check_win32_error() {
            Ok(_) => (),
            Err(_message) => panic!("failed to get monitor information : {_message}"),
        }
    }

    fn poll_event(&self) -> Option<PlatformEvent> {
        let mut event_queue = self.messages.lock().unwrap();
        if let Some(event) = event_queue.pop_front() {
            return Some(event)
        }
        else {
            drop(event_queue);

            //@TODO : make this think cleaner
            unsafe {
                let mut msg = std::mem::zeroed();
                if PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE) != false {
                    TranslateMessage(&mut msg);
                    DispatchMessageW(&mut msg);
                }
            }

            None
        }
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
        Err(error) => panic!("failed to get DPI for monitor {}", error)
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
        primary: info.dwFlags & MONITORINFOF_PRIMARY != 0
    });

    match check_win32_error() {
        Ok(_) => (),
        Err(_message) => panic!("failed to get monitor information : {_message}"),
    }
    BOOL::from(true)
}
