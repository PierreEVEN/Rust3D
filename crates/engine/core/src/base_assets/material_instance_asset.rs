use std::sync::Arc;
use shader_base::BindPoint;
use crate::base_assets::asset::{AssetMetaData, GameAsset};
use crate::gfx::image::GfxImage;
use crate::gfx::image_sampler::ImageSampler;

pub struct MaterialInstanceAsset {
    meta_data: AssetMetaData,
}

impl GameAsset for MaterialInstanceAsset {
    fn save(&self) -> Result<(), String> {
        todo!()
    }

    fn reload(&self) -> Result<(), String> {
        todo!()
    }

    fn meta_data(&self) -> &AssetMetaData {
        &self.meta_data
    }
}

impl MaterialInstanceAsset {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            meta_data: AssetMetaData::new(),
        })
    }

    pub fn bind_texture(&self, _bind_point: &BindPoint, _texture: &Arc<dyn GfxImage>) {
        todo!()
    }

    pub fn bind_sampler(&self, _bind_point: &BindPoint, _texture: &Arc<dyn ImageSampler>) {
        todo!()
    }
}
