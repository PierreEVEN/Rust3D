use std::collections::VecDeque;
use std::mem::size_of;
use std::ptr::null;
use std::rc::Weak;
use std::sync::Arc;
use raw_window_handle::RawWindowHandle;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, GetLastError, HINSTANCE, HWND, LPARAM, NO_ERROR, RECT};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRectEx, CreateWindowExW, DestroyWindow, GET_CLASS_LONG_INDEX, HMENU, LWA_ALPHA, SetClassLongPtrW, SetLayeredWindowAttributes, ShowWindow, SW_MAXIMIZE, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WS_CAPTION, WS_EX_LAYERED, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE};
use maths::rect2d::{Rect2D};
use plateform::{Platform, Monitor, WindowCreateInfos, WindowMessage, Window, WindowFlagBits};

const WIN_CLASS_NAME: &str = "r3d_window";

pub struct MonitorWindows {
    
}

impl MonitorWindows {
    fn new() -> Arc<MonitorWindows> {
        Arc::new(MonitorWindows {
            
        })
    }
}

pub struct WindowWindows {
    
}

impl Window for WindowWindows {
    fn set_geometry(&self, geometry: Rect2D<i32>) {
        todo!()
    }

    fn set_title(&self, title: &str) {
        todo!()
    }

    fn show(&self) {
        todo!()
    }

    fn get_handle(&self) -> RawWindowHandle {
        todo!()
    }

    fn get_geometry(&self) -> Rect2D<i32> {
        todo!()
    }
}

pub struct PlatformWindows {
    windows: Vec<Weak<WindowWindows>>,
    messages: VecDeque<WindowMessage>,
    monitors: Vec<Monitor>,
}

pub fn utf8_to_utf16(str : &str) -> Vec<u16>
{
    str.encode_utf16().chain(Some(0)).collect()
}

impl PlatformWindows {
    pub fn new() -> Arc<PlatformWindows> {
        let platform = Arc::new(PlatformWindows {
            windows: Default::default(),
            messages: VecDeque::new(),
            monitors: Default::default(),
        });

        unsafe {
            // Create dummy window to set platform pointer into the WNDCLASS
            {
                let dummy_window = CreateWindowExW(
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
                    dummy_window,
                    GET_CLASS_LONG_INDEX(0),
                    (platform.as_ref() as *const PlatformWindows) as isize,
                );

                DestroyWindow(dummy_window);
            }
            
        }

        platform.fetch_monitors();
        
        platform
    }
    
    pub fn fetch_monitors(&self) {

        unsafe {
            EnumDisplayMonitors(
                HDC::default(),
                null(),
                Some(enum_display_monitors_callback),
                LPARAM((&self.monitors as *const _) as isize),
            );
        }
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
    GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);

    let mut monitors = (userdata.0 as *mut Vec<Monitor>)
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
    });

    BOOL::from(true)
}

impl Platform for PlatformWindows {
    fn create_window(&self, create_infos: WindowCreateInfos) -> Result<Arc<dyn Window>, ()> {

        let ex_style = WS_EX_LAYERED;
        let mut style = WINDOW_STYLE::default();

        if create_infos.window_flags.contains(WindowFlagBits::Borderless) {
            style |= WS_VISIBLE | WS_POPUP;
        } else {
            style |= WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
        }

        if create_infos.window_flags.contains(WindowFlagBits::Resizable) {
            style |= WS_THICKFRAME;
        }

        // Rect must be ajusted since Win32 api include window decoration in the width/height
        let mut initial_rect = RECT {
            left: 0,
            top: 0,
            right: (create_infos.geometry.max_x - create_infos.geometry.min_x) as i32,
            bottom: (create_infos.geometry.max_y - create_infos.geometry.min_y) as i32,
        };

        unsafe {
            AdjustWindowRectEx(&mut initial_rect, style, false, ex_style);
            let hwnd = CreateWindowExW(
                ex_style,
                PCWSTR(utf8_to_utf16(WIN_CLASS_NAME).as_ptr()),
                PCWSTR(utf8_to_utf16(create_infos.name.as_str()).as_ptr()),
                style,
                create_infos.geometry.min_x as i32 + initial_rect.left,
                create_infos.geometry.min_y as i32 + initial_rect.top,
                initial_rect.right - initial_rect.left,
                initial_rect.bottom - initial_rect.top,
                HWND::default(),
                HMENU::default(),
                HINSTANCE::default(),
                null(),
            );

            if GetLastError() != NO_ERROR {
                return Err(());
            }

            SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);

            ShowWindow(
                hwnd,
                if create_infos.window_flags.contains(WindowFlagBits::Maximized) {
                    SW_MAXIMIZE
                } else {
                    SW_SHOW
                },
            );
            
            let window = Arc::new(WindowWindows {}); //::new(hwnd, width, height, x, y, style, ex_style);
            //self.windows.insert(hwnd.into(), Arc::downgrade(&window));
            return Ok(window);
        };
    }

    fn monitor_count(&self) -> usize {
        todo!()
    }

    fn get_monitor(&self, index: usize) -> Monitor {
        todo!()
    }
}