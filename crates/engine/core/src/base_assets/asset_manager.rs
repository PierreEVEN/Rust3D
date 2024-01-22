
use crate::base_assets::material_asset::MaterialAssetFactory;
use crate::base_assets::mesh_asset::MeshAssetFactory;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::base_assets::asset::{AssetFactory, GameAsset};
use crate::base_assets::asset_id::AssetID;
use crate::base_assets::asset_type_id::AssetTypeID;

pub struct AssetManager {
    factories: RwLock<HashMap<AssetTypeID, Arc<dyn AssetFactory>>>,
    _assets: RwLock<HashMap<AssetID, Arc<dyn GameAsset>>>,
}

impl Default for AssetManager {
    fn default() -> Self {
        let asset_manager = Self {
            _assets: RwLock::default(),
            factories: RwLock::default(),
        };

        asset_manager.register_factory(MaterialAssetFactory::new());
        asset_manager.register_factory(MeshAssetFactory::new());

        asset_manager
    }
}

impl AssetManager {
    pub fn register_factory(&self, factory: Arc<dyn AssetFactory>) {
        self.factories
            .write()
            .unwrap()
            .insert(factory.asset_id(), factory);
    }

    pub fn find_factory(&self, type_id: AssetTypeID) -> Result<Arc<dyn AssetFactory>, String> {
        match self.factories.read().unwrap().get(&type_id) {
            None => Err("failed to find factory".to_string()),
            Some(factory) => Ok(factory.clone()),
        }
    }
}
