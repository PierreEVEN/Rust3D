use gfx::image::GfxImage;
use gfx::image_sampler::ImageSampler;
use std::sync::Arc;
use shader_base::BindPoint;

use crate::asset::{AssetMetaData, GameAsset};

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
