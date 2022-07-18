use std::rc::Weak;
use std::sync::Arc;
use enumflags2::{bitflags, BitFlags};
use maths::rect2d::{Rect2D};
use raw_window_handle::RawWindowHandle;

#[derive(Copy, Clone)]
pub struct Monitor {
    pub bounds: Rect2D<i32>,
    pub work_bounds: Rect2D<i32>,
    pub dpi: f32,
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum WindowFlagBits {
    Maximized = 1 << 1,
    Borderless = 1 << 2,
    Resizable = 1 << 3,
    Transparent = 1 << 4,
}
pub type WindowFlags = BitFlags<WindowFlagBits>;

#[derive(Clone)]
pub struct WindowCreateInfos {
    pub name: String,
    pub geometry: Rect2D<u32>,
    pub window_flags: WindowFlags,
}

pub enum WindowMessage {
    WindowClosed(Weak<dyn Window>),
    WindowResized(Weak<dyn Window>, u32, u32),
}

pub trait Window {
    fn set_geometry(&self, geometry: Rect2D<i32>);
    fn set_title(&self, title: &str);
    fn show(&self);

    fn get_handle(&self) -> RawWindowHandle;
    fn get_geometry(&self) -> Rect2D<i32>;
}

pub trait Platform {
    
    fn create_window(&self, create_infos: WindowCreateInfos) -> Result<Arc<dyn Window>, ()>;
    
    fn monitor_count (&self) -> usize;
    fn get_monitor(&self, index:usize) -> Monitor;
}
