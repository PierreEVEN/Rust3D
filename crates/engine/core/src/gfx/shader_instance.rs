use std::sync::Arc;
use shader_base::types::GfxCast;
use crate::gfx::material::MaterialResourcePool;

pub struct ShaderInstanceCreateInfos {
    pub resources: Arc<MaterialResourcePool>,
}

pub trait ShaderInstance: GfxCast {
}

impl dyn ShaderInstance {
    pub fn cast<U: ShaderInstance + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}
