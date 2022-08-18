use std::sync::{Arc};
use maths::rect2d::{RectI32};
use enumflags2::{bitflags, BitFlags};
use raw_window_handle::RawWindowHandle;

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum WindowFlagBits {
    Maximized = 1 << 1,
    Borderless = 1 << 2,
    Resizable = 1 << 3,
}
pub type WindowFlags = BitFlags<WindowFlagBits>;

#[derive(Clone)]
pub struct WindowCreateInfos {
    pub name: String,
    pub geometry: RectI32,
    pub window_flags: WindowFlags,
    pub background_alpha: u8,
}

pub enum PlatformEvent {
    WindowClosed(Arc<dyn Window>),
    WindowResized(Arc<dyn Window>, u32, u32),
}

pub trait Window {
    fn set_geometry(&self, geometry: RectI32);
    fn get_geometry(&self) -> RectI32;
    fn set_title(&self, title: &str);
    fn get_title(&self) -> String;
    fn show(&self);
    fn set_background_alpha(&self, alpha: u8);
    fn get_background_alpha(&self) -> u8;
    fn get_handle(&self) -> RawWindowHandle;
}
