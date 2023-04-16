use std::sync::{Arc, RwLock, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;

use gfx::{GfxRef};
use plateform::Platform;

use crate::asset_manager::AssetManager;

pub struct Engine {
    pub asset_manager: Arc<AssetManager>,
    pub platform: Arc<dyn Platform>,
    pub gfx: GfxRef,
    b_run: AtomicBool,
}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

lazy_static!(
    static ref ENGINE_REFERENCE: RwLock<Option<Weak<Engine>>> = RwLock::new(None);
);

impl Engine {
    pub fn new<PlatformT: Platform + 'static>(platform: Arc<PlatformT>, gfx: GfxRef) -> Arc<Self> {
        gfx.set_physical_device(gfx.find_best_suitable_physical_device().expect("there is no suitable GPU available"));

        let engine = Arc::new(Self {
            asset_manager: AssetManager::new(&gfx),
            platform,
            gfx,
            b_run: AtomicBool::from(true)
        });
        *ENGINE_REFERENCE.write().unwrap() = Some(Arc::downgrade(&engine));
        engine
    }
    
    pub fn get<'l>() -> &'l Self {
        match &*ENGINE_REFERENCE.read().unwrap() {
            None => { logger::fatal!("engine is not valid in the current context"); }
            Some(engine) => { unsafe { engine.as_ptr().as_ref().unwrap() } }
        }
    }
    
    pub fn shutdown(&self) {
        self.b_run.store(false, Ordering::Release);
    }
    
    pub fn run(&self) -> bool {
        self.b_run.load(Ordering::Acquire)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        logger::info!("shutting down engine...");
        *ENGINE_REFERENCE.write().unwrap() = None;
    }
}