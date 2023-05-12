pub mod input_system;
pub mod window;

use crate::input_system::InputManager;
use crate::window::{Window, WindowCreateInfos};
use maths::rect2d::Rect2D;
use std::sync::{Weak};

#[derive(Copy, Clone)]
pub struct Monitor {
    pub bounds: Rect2D<i32>,
    pub work_bounds: Rect2D<i32>,
    pub dpi: f32,
    pub primary: bool,
}

impl ToString for Monitor {
    fn to_string(&self) -> String {
        format!(
            "\
        primary = {}\n\
        bounds : {}\n\
        work bounds : {}\n\
        dpi : {}",
            self.primary,
            self.bounds.to_string(),
            self.work_bounds.to_string(),
            self.dpi
        )
    }
}

#[derive(Debug, Clone)]
pub struct WindowCreationError;

pub trait Platform {
    fn create_window(
        &self,
        create_infos: WindowCreateInfos,
    ) -> Result<Weak<dyn Window>, WindowCreationError>;
    fn monitor_count(&self) -> usize;
    fn get_monitor(&self, index: usize) -> Monitor;
    fn collect_monitors(&self);
    fn poll_events(&self);
    fn input_manager(&self) -> &InputManager;
}
