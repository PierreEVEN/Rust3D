use std::any::Any;
use std::hash::{Hash, Hasher};

use enumflags2::{bitflags, BitFlags};

use maths::rect2d::RectI32;

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

#[derive(Clone)]
pub enum PlatformEvent {
    WindowClosed,
    WindowResized(u32, u32),
}

impl PlatformEvent {
    pub fn id(&self) -> u16 {
        match self {
            PlatformEvent::WindowClosed => { 1 }
            PlatformEvent::WindowResized(_, _) => { 2 }
        }
    }
}

impl Hash for PlatformEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(self.id())
    }
}

impl PartialEq for PlatformEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for PlatformEvent {}

pub type WindowEventDelegate = Box<dyn FnMut(&PlatformEvent)>;

pub trait WindowCast: 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> WindowCast for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait Window : WindowCast {
    fn set_geometry(&self, geometry: RectI32);
    fn get_geometry(&self) -> RectI32;
    fn set_title(&self, title: &str);
    fn get_title(&self) -> String;
    fn show(&self);
    fn set_background_alpha(&self, alpha: u8);
    fn get_background_alpha(&self) -> u8;
    fn bind_event(&self, event_type: PlatformEvent, delegate: WindowEventDelegate);
}

impl dyn Window {
    pub fn cast<U: Window + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}
