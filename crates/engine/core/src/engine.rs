use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};

use gfx::{GfxRef};
use plateform::Platform;

use crate::asset_manager::AssetManager;

pub struct Engine {
    pub asset_manager: Arc<AssetManager>,
    pub platform: Arc<dyn Platform>,
    pub gfx: GfxRef,
    b_run: AtomicBool,
}

static mut ENGINE_REFERENCE: Option<Arc<Engine>> = None;

impl Engine {
    pub fn new<PlatformT: Platform + 'static>(platform: Arc<PlatformT>, gfx: GfxRef) -> Arc<Self> {
        gfx.set_physical_device(gfx.find_best_suitable_physical_device().expect("there is no suitable GPU available"));

        let engine = Arc::new(Self {
            asset_manager: AssetManager::new(&gfx),
            platform,
            gfx,
            b_run: AtomicBool::from(true)
        });
        unsafe { ENGINE_REFERENCE = Some(engine.clone()); }
        engine
    }

    pub fn get() -> Arc<Self> {
        unsafe {
            match &ENGINE_REFERENCE {
                None => { logger::fatal!("engine is not valid in the current context"); }
                Some(engine) => { engine.clone() }
            }
        }
    }
    
    pub fn shutdown(&self) {
        self.b_run.store(false, Ordering::Release);
    }
    
    pub fn run(&self) -> bool {
        self.b_run.load(Ordering::Acquire)
    }
}