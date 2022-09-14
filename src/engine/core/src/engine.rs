use std::sync::{Arc, RwLock};

use gfx::{GfxRef};
use plateform::Platform;

use crate::asset_manager::AssetManager;

pub struct Engine {
    pub asset_manager: Arc<AssetManager>,
    pub platform: Arc<dyn Platform>,
    pub gfx: GfxRef,
}

static mut ENGINE_REFERENCE: Option<Arc<Engine>> = None;

impl Engine {
    pub fn new<PlatformT: Platform + 'static>(platform: Arc<PlatformT>, gfx: GfxRef) -> Arc<Self> {
        gfx.set_physical_device(gfx.find_best_suitable_physical_device().expect("there is no suitable GPU available"));

        let engine = Arc::new(Self {
            asset_manager: AssetManager::new(&gfx),
            platform,
            gfx,
        });
        unsafe { ENGINE_REFERENCE = Some(engine.clone()); }
        engine
    }

    pub fn get() -> Arc<Self> {
        unsafe {
            match &ENGINE_REFERENCE {
                None => { panic!("engine is not valid in the current context"); }
                Some(engine) => { engine.clone() }
            }
        }
    }
}