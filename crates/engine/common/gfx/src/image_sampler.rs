use shader_base::types::GfxCast;

pub struct SamplerCreateInfos {}

pub trait ImageSampler: GfxCast {}

impl dyn ImageSampler {
    pub fn cast<U: ImageSampler + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}
