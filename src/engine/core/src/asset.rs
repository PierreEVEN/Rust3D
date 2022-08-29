use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::asset_manager::AssetManager;
use crate::asset_type_id::AssetTypeID;

pub trait GameAsset {
    fn save(&self) -> Result<(), String>;
    fn reload(&self) -> Result<(), String>;
    fn meta_data(&self) -> &AssetMetaData;
}

pub trait AssetFactory {
    fn instantiate_from_asset_path(&self, path: &Path) -> Arc<dyn GameAsset>;
    fn asset_id(&self) -> AssetTypeID;
}

pub struct AssetMetaData {
    pub asset_manager: Arc<AssetManager>,
    name: RwLock<String>,
    save_path: RwLock<Option<String>>,
}

impl AssetMetaData {
    pub fn new(asset_manager: &Arc<AssetManager>) -> Self {
        Self {
            asset_manager: asset_manager.clone(),
            name: RwLock::default(),
            save_path: RwLock::default(),
        }
    }
    
    pub fn is_transient(&self) -> bool {
        match *self.save_path.read().unwrap() {
            None => { true }
            Some(_) => { false }
        }
    }

    pub fn set_save_path(&self, path: &Path) {
        *self.save_path.write().unwrap() = Some(path.to_str().unwrap().to_string());
    }

    pub fn get_save_path(&self) -> Option<String> {
        match &*self.save_path.read().unwrap() {
            None => { None }
            Some(path) => { Some(path.clone()) }
        }
    }

    pub fn get_name(&self) -> String {
        self.name.read().unwrap().clone()
    }

    pub fn set_name(&self, name: String) {
        *self.name.write().unwrap() = name
    }
}