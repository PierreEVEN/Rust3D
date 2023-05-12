use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use gfx::GfxInterface;
use plateform::Platform;

use crate::asset_manager::AssetManager;
use crate::world::World;

pub struct DeltaSeconds {
    last: std::time::Instant,
    delta_seconds: f64,
    limit: Option<f64>,
}

impl DeltaSeconds {
    pub fn new(limit: Option<f64>) -> Self {
        Self {
            last: std::time::Instant::now(),
            delta_seconds: 0.0,
            limit,
        }
    }

    pub fn new_frame(&mut self) {
        self.delta_seconds = match self.limit {
            None => (std::time::Instant::now() - self.last).as_secs_f64(),
            Some(value) => value.min((std::time::Instant::now() - self.last).as_secs_f64()),
        };
        self.last = std::time::Instant::now();
    }

    pub fn current(&self) -> f64 {
        self.delta_seconds
    }
}

pub trait App {
    fn pre_initialize(&mut self, builder: &mut Builder);
    fn initialized(&mut self);

    fn new_frame(&mut self, delta_seconds: f64);

    fn request_shutdown(&self);
    fn stopped(&self);
}

pub struct Builder {
    pub platform: Box<dyn FnMut() -> Box<dyn Platform>>,
    pub gfx: Box<dyn FnMut() -> Box<dyn GfxInterface>>,
    pub asset_manager: Box<dyn FnMut() -> AssetManager>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            platform: Box::new(|| backend_launcher::backend::spawn_platform()),
            gfx: Box::new(|| {
                let mut gfx = backend_launcher::backend::spawn_gfx();
                gfx.pre_init();
                gfx.init();
                gfx.set_physical_device(
                    gfx.find_best_suitable_physical_device()
                        .expect("there is no suitable GPU available"),
                );
                gfx
            }),
            asset_manager: Box::new(|| AssetManager::default()),
        }
    }
}

pub struct Engine {
    asset_manager: MaybeUninit<AssetManager>,
    platform: MaybeUninit<Box<dyn Platform>>,
    gfx: MaybeUninit<Box<dyn GfxInterface>>,
    app: Box<dyn App>,

    pre_initialized: AtomicBool,
    initialized: AtomicBool,
    is_stopping: AtomicBool,

    worlds: RwLock<Vec<Arc<World>>>,
    game_delta: DeltaSeconds,
}

static mut ENGINE_INSTANCE: *mut Engine = std::ptr::null_mut();

impl Engine {
    pub fn new<GamemodeT: App + Default + 'static>() -> Self {
        logger::init!();

        unsafe {
            assert!(
                ENGINE_INSTANCE.is_null(),
                "an other engine instance is already running"
            )
        }

        let engine = Self {
            asset_manager: MaybeUninit::uninit(),
            platform: MaybeUninit::uninit(),
            gfx: MaybeUninit::uninit(),
            app: Box::<GamemodeT>::default(),
            pre_initialized: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
            is_stopping: AtomicBool::new(false),
            worlds: Default::default(),
            game_delta: DeltaSeconds::new(None),
        };
        unsafe {
            ENGINE_INSTANCE = &engine as *const Engine as *mut Engine;
        }
        engine
    }

    pub fn start(&mut self) {
        // ENTER ENGINE PRE-INITIALIZATION
        assert!(
            !self.pre_initialized.load(Ordering::SeqCst),
            "Engine.start() have already been called !"
        );
        self.pre_initialized.store(true, Ordering::SeqCst);
        logger::info!("Engine pre-initialization");

        let mut builder = Builder::default();
        self.app.pre_initialize(&mut builder);

        self.platform = MaybeUninit::new((*builder.platform)());
        self.gfx = MaybeUninit::new((*builder.gfx)());
        self.asset_manager = MaybeUninit::new((*builder.asset_manager)());

        // FINISHED PRE-INITIALIZATION
        self.initialized.store(true, Ordering::SeqCst);
        logger::info!("Engine started");

        self.app.initialized();

        self.engine_loop();
    }

    pub fn shutdown(&self) {
        self.check_validity();
        self.app.request_shutdown();
        self.is_stopping.store(true, Ordering::Release);
    }

    pub fn check_validity(&self) {
        assert!(
            self.pre_initialized.load(Ordering::SeqCst),
            "Engine is not initialized ! Please call Engine.start() before"
        );
        assert!(self.initialized.load(Ordering::SeqCst), "Engine is not fully initialized ! Please wait full engine initialization before using Engine::get()");
    }

    pub fn get() -> &'static Self {
        unsafe {
            assert!(
                !ENGINE_INSTANCE.is_null(),
                "Engine is not available there ! Please call Engine::init() before"
            )
        }
        unsafe {
            let engine = ENGINE_INSTANCE.as_ref().unwrap();
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

    pub fn new_world(&self) -> Arc<World> {
        let world = Arc::new(World::new());
        self.worlds.write().unwrap().push(world.clone());
        world
    }

    fn engine_loop(&mut self) {
        while !self.is_stopping.load(Ordering::SeqCst) {
            self.game_delta.new_frame();
            self.platform().poll_events();
            self.app.new_frame(self.game_delta.current());
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.app.stopped();

        // Started de-initialization
        self.initialized.store(false, Ordering::SeqCst);
        unsafe {
            self.asset_manager.assume_init_drop();
            self.gfx.assume_init_drop();
            self.platform.assume_init_drop();
        }

        // Now de-initialized
        self.pre_initialized.store(false, Ordering::SeqCst);
        unsafe { ENGINE_INSTANCE = std::ptr::null_mut() }
        logger::info!("closed engine");
    }
}
