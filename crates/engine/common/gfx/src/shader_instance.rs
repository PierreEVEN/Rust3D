use std::collections::HashMap;
use std::sync::Arc;
use shader_base::BindPoint;
use shader_base::spirv_reflector::DescriptorBinding;
use shader_base::types::GfxCast;

use crate::{GfxImage, ImageSampler};

pub struct ShaderInstanceCreateInfos {
    pub bindings: HashMap<BindPoint, DescriptorBinding>,
}

pub trait ShaderInstance: GfxCast {
}

impl dyn ShaderInstance {
    pub fn cast<U: ShaderInstance + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}
