use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use gfx::GfxRef;
use crate::asset::{AssetFactory, GameAsset};
use crate::asset_id::AssetID;
use crate::asset_type_id::AssetTypeID;
use crate::base_assets::material_asset::MaterialAssetFactory;
use crate::base_assets::mesh_asset::MeshAssetFactory;

pub struct AssetManager {
    factories: RwLock<HashMap<AssetTypeID, Arc<dyn AssetFactory>>>,
    _assets: RwLock<HashMap<AssetID, Arc<dyn GameAsset>>>,
    gfx: GfxRef,
}

impl AssetManager {
    pub fn new(gfx: &GfxRef) -> Arc<Self> {
        let asset_manager = Arc::new(Self {
            _assets: RwLock::default(),
            factories: RwLock::default(),
            gfx: gfx.clone(),
        });

        asset_manager.register_factory(MaterialAssetFactory::new());
        asset_manager.register_factory(MeshAssetFactory::new());

        asset_manager
    }

    pub fn register_factory(&self, factory: Arc<dyn AssetFactory>) {
        self.factories.write().unwrap().insert(factory.asset_id(), factory);
    }

    pub fn find_factory(&self, type_id: AssetTypeID) -> Result<Arc<dyn AssetFactory>, String> {
        match self.factories.read().unwrap().get(&type_id) {
            None => { Err("failed to find factory".to_string()) }
            Some(factory) => { Ok(factory.clone()) }
        }
    }
    
    pub fn graphics(&self) -> &GfxRef {
        &self.gfx
    }
}