use std::sync::Arc;
use shader_base::BindPoint;
use shader_base::spirv_reflector::DescriptorBinding;

use crate::{GfxCast, GfxImage, ImageSampler};

pub struct ShaderInstanceCreateInfos {
    pub bindings: Vec<DescriptorBinding>,
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
