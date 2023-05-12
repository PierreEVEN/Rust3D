use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::asset::{AssetFactory, AssetMetaData, GameAsset};
use crate::asset_manager::AssetManager;
use crate::asset_type_id::AssetTypeID;
use crate::base_assets::material_asset::MaterialAsset;

pub struct MeshAsset {
    material: RwLock<Arc<MaterialAsset>>,
    _meta_data: AssetMetaData,
}

impl MeshAsset {
    pub fn new(_asset_manager: &Arc<AssetManager>) -> Arc<MeshAsset> {
        todo!()
        /*
        Arc::new(Self {
            _meta_data: AssetMetaData::new(asset_manager),
            material: todo!()
        })
         */
    }

    pub fn set_material(&self, material: &Arc<MaterialAsset>) {
        *self.material.write().unwrap() = material.clone();
    }

    pub fn get_material(&self) -> Arc<MaterialAsset> {
        self.material.read().unwrap().clone()
    }
}

impl GameAsset for MeshAsset {
    fn save(&self) -> Result<(), String> {
        todo!()
    }

    fn reload(&self) -> Result<(), String> {
        todo!()
    }

    fn meta_data(&self) -> &AssetMetaData {
        &self._meta_data
    }
}

pub struct MeshAssetFactory {}

impl MeshAssetFactory {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}

impl AssetFactory for MeshAssetFactory {
    fn instantiate_from_asset_path(&self, _path: &Path) -> Arc<dyn GameAsset> {
        todo!()
    }

    fn asset_id(&self) -> AssetTypeID {
        AssetTypeID::from("mesh")
    }
}
