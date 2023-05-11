use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, Ordering};
use gfx::{GfxInterface};
use plateform::Platform;

use crate::asset_manager::AssetManager;

pub trait App {

}

pub trait EngineComponent {
    fn pre_init(&self, engine: &Engine);
    fn init(&self, engine: &Engine);
    fn de_init(&self, engine: &Engine);
}

pub struct Engine {
    asset_manager: MaybeUninit<AssetManager>,
    platform: MaybeUninit<Box<dyn Platform>>,
    gfx: MaybeUninit<Box<dyn GfxInterface>>,
    
    initializer: EngineInit,
    
    pre_initialized: AtomicBool,
    initialized: AtomicBool,
    is_stopping: AtomicBool,
}

struct EnginePtr(*mut Engine);
unsafe impl Sync for EnginePtr {}
static mut ENGINE_INSTANCE: EnginePtr = EnginePtr(std::ptr::null_mut());

struct EngineInit {
    pub platform: Box<dyn FnMut() -> Box<dyn Platform>>,
    pub gfx: Box<dyn FnMut() -> Box<dyn GfxInterface>>,
    pub asset_manager: Box<dyn FnMut() -> AssetManager>,    
}

impl Default for EngineInit {
    fn default() -> Self {
        Self {
            platform: Box::new(|| {
                backend_launcher::backend::spawn_platform()
            }),
            gfx: Box::new(|| {
                backend_launcher::backend::spawn_gfx()
            }),
            asset_manager: Box::new(|| {
                AssetManager::default()
            }),
        }
    }
}


impl Engine {
    pub fn new<GamemodeT: App + 'static>() -> Self {
        
        logger::init!();
        
        unsafe { assert!(ENGINE_INSTANCE.0.is_null(), "an other engine instance is already running") }
        
        let engine = Self {
            asset_manager: MaybeUninit::uninit(),
            platform: MaybeUninit::uninit(),
            gfx: MaybeUninit::uninit(),
            initializer: EngineInit::default(),            
            pre_initialized: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
            is_stopping: AtomicBool::new(false),
        };
        unsafe { ENGINE_INSTANCE.0 = &engine as *const Engine as *mut Engine; }
        engine
    }

    pub fn start(&mut self) {
        // ENTER ENGINE PRE-INITIALIZATION
        assert!(!self.pre_initialized.load(Ordering::SeqCst), "Engine.start() have already been called !");
        self.pre_initialized.store(true, Ordering::SeqCst);
        logger::info!("Engine pre-initialization");
        
        self.platform = MaybeUninit::new((*self.initializer.platform)());
        self.gfx = MaybeUninit::new((*self.initializer.gfx)());
        self.asset_manager = MaybeUninit::new((*self.initializer.asset_manager)());
        
        //unsafe { self.gfx.assume_init_ref().set_physical_device(self.gfx.assume_init_ref().find_best_suitable_physical_device().expect("there is no suitable GPU available")); }
        
        // FINISHED PRE-INITIALIZATION
        self.initialized.store(true, Ordering::SeqCst);
        logger::info!("Engine started");
    }
    
    pub fn stop(&self) {
        self.check_validity();
        self.is_stopping.store(true, Ordering::Release);
    }

    pub fn check_validity(&self) {
        assert!(self.pre_initialized.load(Ordering::SeqCst), "Engine is not initialized ! Please call Engine.start() before");
        assert!(self.initialized.load(Ordering::SeqCst), "Engine is not fully initialized ! Please wait full engine initialization before using Engine::get()");
    }
    
    pub fn get() -> &'static Self {
        unsafe { assert!(!ENGINE_INSTANCE.0.is_null(), "Engine is not available there ! Please call Engine::init() before") }
        unsafe {
            let engine = ENGINE_INSTANCE.0.as_ref().unwrap();
            engine.check_validity();
            engine
        }
    }
    
    pub fn gfx(&self) -> &dyn GfxInterface {
        self.check_validity();
        unsafe { self.gfx.assume_init_ref().as_ref() }
    }
    pub fn platform(&self) -> &dyn Platform {
        self.check_validity();
        unsafe { self.platform.assume_init_ref().as_ref() }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Started de-initialization
        self.initialized.store(false, Ordering::SeqCst);
        
        self.asset_manager = MaybeUninit::uninit();
        self.gfx = MaybeUninit::uninit();
        self.platform = MaybeUninit::uninit();
        
        // Now de-initialized
        self.pre_initialized.store(false, Ordering::SeqCst);
        unsafe { ENGINE_INSTANCE.0 = std::ptr::null_mut() }
        logger::info!("closed engine");
    }
}