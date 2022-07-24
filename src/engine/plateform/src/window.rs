use std::sync::{Arc, Mutex};
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
    Transparent = 1 << 4,
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
    WindowClosed(Arc<Mutex<dyn Window>>),
    WindowResized(Arc<Mutex<dyn Window>>, u32, u32),
}

pub trait Window {
    fn set_geometry(&mut self, geometry: RectI32);
    fn get_geometry(&self) -> RectI32;
    fn set_title(&mut self, title: &str);
    fn get_title(&self) -> &str;
    fn show(&self);
    fn set_background_alpha(&mut self, alpha: u8);
    fn get_background_alpha(&self) -> u8;
    fn get_handle(&self) -> RawWindowHandle;
}
