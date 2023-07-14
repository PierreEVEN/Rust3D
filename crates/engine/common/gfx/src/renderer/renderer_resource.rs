use std::hash::{Hash, Hasher};
use crate::types::{BackgroundColor, PixelFormat};

#[derive(Clone)]
pub struct PassResource {
    pub name: String,
    pub clear_value: BackgroundColor,
    pub format: PixelFormat,
}

impl Hash for PassResource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.format.hash(state);
    }
}

impl PartialEq for PassResource {
    fn eq(&self, other: &Self) -> bool {
        self.format == other.format
    }
}
