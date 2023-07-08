use std::hash::{Hash, Hasher};
use crate::renderer::renderer_resource::Resource::{Color, Depth};
use crate::types::{ClearValues, PixelFormat};

#[derive(Clone)]
pub struct ResourceColor {
    pub name: String,
    pub clear_value: ClearValues,
    pub image_format: PixelFormat,
}

#[derive(Clone)]
pub struct ResourceDepth {
    pub name: String,
    pub clear_value: ClearValues,
    pub image_format: PixelFormat,
}

#[derive(Clone)]
pub enum Resource {
    Color(ResourceColor),
    Depth(ResourceDepth),
}

impl Resource {
    pub fn name(&self) -> &String {
        match self {
            Color(color) => {&color.name}
            Depth(depth) => {&depth.name}
        }
    }

    pub fn format(&self) -> &PixelFormat {
        match self {
            Color(color) => {&color.image_format}
            Depth(depth) => {&depth.image_format}
        }
    }
}

impl Hash for Resource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Color(color_resource) => {
                state.write_u8(128);
                color_resource.image_format.hash(state);
            }
            Depth(depth_resource) => {
                state.write_u8(255);
                depth_resource.image_format.hash(state);
            }
        }
    }
}

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Color(color) => {
                if let Color(other_color) = other { color.image_format == other_color.image_format } else { false }
            }
            Depth(depth) => {
                if let Depth(other_depth) = other { depth.image_format == other_depth.image_format } else { false }
            }
        }
    }
}
