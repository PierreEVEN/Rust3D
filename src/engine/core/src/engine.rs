use std::sync::Arc;

use gfx::{GfxRef};
use plateform::Platform;

use crate::asset_manager::AssetManager;

pub struct Engine {
    pub asset_manager: Arc<AssetManager>,
    pub platform: Arc<dyn Platform>,
    pub gfx: GfxRef,
}

impl Engine {
    pub fn new<PlatformT: Platform + 'static>(platform: Arc<PlatformT>, gfx: GfxRef) -> Arc<Self> {

        gfx.set_physical_device(gfx.find_best_suitable_physical_device().expect("there is no suitable GPU available"));
        
        Arc::new(Self {
            asset_manager: AssetManager::new(&gfx),
            platform,
            gfx,
        })
    }
}