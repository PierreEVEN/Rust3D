use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::shader::DescriptorBinding;
use crate::{GfxCast, GfxImage, ImageSampler};

pub struct ShaderInstanceCreateInfos {
    pub bindings: Vec<DescriptorBinding>,
}

#[derive(Clone)]
pub struct BindPoint {
    pub name: String,
}

impl Hash for BindPoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes())
    }
}

impl PartialEq<Self> for BindPoint {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for BindPoint {}

impl BindPoint {
    pub fn new(name: &str) -> BindPoint {
        BindPoint {
            name: name.to_string(),
        }
    }
}

pub trait ShaderInstance: GfxCast {
    fn bind_texture(&self, bind_point: &BindPoint, texture: &Arc<dyn GfxImage>);
    fn bind_sampler(&self, bind_point: &BindPoint, texture: &Arc<dyn ImageSampler>);
}

impl dyn ShaderInstance {
    pub fn cast<U: ShaderInstance + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}
