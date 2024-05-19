use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Condvar, Mutex, RwLock, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;

use logger::{fatal, info};
use plateform::Platform;
use plateform::window::Window;

use crate::base_assets::asset_manager::AssetManager;
use crate::gfx::{Gfx, GfxInterface};
use crate::gfx::renderer::renderer::Renderer;
use crate::gfx::surface::{Frame, GfxSurface};
use crate::resource::allocator::ResourceAllocator;
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
            platform: Box::new(|| { panic!("Platform has not been defined") }),
            gfx: Box::new(|| { panic!("Gfx backend has not been defined") }),
            asset_manager: Box::new(AssetManager::default),
        }
    }
}

pub struct Engine {
    asset_manager: MaybeUninit<AssetManager>,
    resource_allocator: ResourceAllocator,
    platform: MaybeUninit<Box<dyn Platform>>,
    gfx: MaybeUninit<Box<dyn GfxInterface>>,
    app: Box<dyn App>,
    pub engine_number: u64,

    pre_initialized: AtomicBool,
    initialized_lock: Mutex<bool>,
    initialized_ready: Condvar,
    is_stopping: AtomicBool,

    worlds: RwLock<Vec<Arc<World>>>,
    game_delta: DeltaSeconds,
}

static mut ENGINE_INSTANCE: *mut Engine = std::ptr::null_mut();

pub struct EngineRef {}

impl EngineRef {
    pub fn new(engine: Engine) -> Self {
        unsafe {
            if !ENGINE_INSTANCE.is_null() {
                fatal!("Cannot initialize EngineRef : Engine is already instanced");
            }
        }
        unsafe { ENGINE_INSTANCE = Box::into_raw(Box::new(engine)); }
        Self {}
    }
}

impl Drop for EngineRef {
    fn drop(&mut self) {
        unsafe {
            let _boxed = Box::from_raw(ENGINE_INSTANCE);
            ENGINE_INSTANCE = std::ptr::null_mut()
        }
    }
}

impl Deref for EngineRef {
    type Target = Engine;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(ENGINE_INSTANCE as *const Engine) }
    }
}

impl DerefMut for EngineRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(ENGINE_INSTANCE) }
    }
}

impl Engine {
    pub fn set_ptr(engine: &Engine) {
        unsafe {
            ENGINE_INSTANCE = engine as *const Engine as *mut Engine;
        }
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new<GamemodeT: App + 'static>(app: GamemodeT) -> EngineRef {
        logger::init!();

        unsafe {
            assert!(
                ENGINE_INSTANCE.is_null(),
                "an other engine instance is already running"
            )
        }

        let engine = Self {
            asset_manager: MaybeUninit::uninit(),
            resource_allocator: ResourceAllocator::default(),
            platform: MaybeUninit::uninit(),
            gfx: MaybeUninit::uninit(),
            app: Box::new(app),
            engine_number: 1234567890,
            pre_initialized: AtomicBool::new(false),
            initialized_lock: Mutex::new(false),
            initialized_ready: Default::default(),
            is_stopping: AtomicBool::new(false),
            worlds: Default::default(),
            game_delta: DeltaSeconds::new(None),
        };
        EngineRef::new(engine)
    }

    pub fn app(&self) -> &dyn App {
        self.app.as_ref()
    }

    pub fn start(&mut self) {
        // ENTER ENGINE PRE-INITIALIZATION
        assert!(
            !self.pre_initialized.load(Ordering::SeqCst),
            "Engine.start() have already been called !"
        );
        self.pre_initialized.store(true, Ordering::SeqCst);
        info!("Engine pre-initialization");

        let mut builder = Builder::default();
        self.app.pre_initialize(&mut builder);

        self.platform = MaybeUninit::new((*builder.platform)());
        self.gfx = MaybeUninit::new((*builder.gfx)());
        self.asset_manager = MaybeUninit::new((*builder.asset_manager)());

        // FINISHED PRE-INITIALIZATION
        *self.initialized_lock.lock().unwrap() = true;
        self.initialized_ready.notify_all();
        info!("Engine started");
        self.app.initialized();

        Gfx::get().launch_render_threads();
        self.engine_loop();
    }

    pub fn shutdown(&self) {
        self.check_validity();
        self.app.request_shutdown();
        self.is_stopping.store(true, Ordering::Release);
    }

    pub fn wait_initialization(&self) {
        let mut initialized = self.initialized_lock.lock().unwrap();
        while !*initialized {
            initialized = self.initialized_ready.wait(initialized).unwrap();
        }
    }

    pub fn check_validity(&self) {
        assert!(
            self.pre_initialized.load(Ordering::SeqCst),
            "Engine is not initialized ! Please call Engine.start() before"
        );
        assert!(*self.initialized_lock.lock().unwrap(), "Engine is not fully initialized ! Please wait full engine initialization before using Engine::get()");
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

    pub fn get_mut() -> &'static mut Self {
        unsafe {
            assert!(
                !ENGINE_INSTANCE.is_null(),
                "Engine is not available there ! Please call Engine::init() before"
            )
        }
        unsafe {
            let engine = ENGINE_INSTANCE.as_mut().unwrap();
            engine.check_validity();
            engine
        }
    }

    pub fn resource_allocator(&self) -> &ResourceAllocator {
        &self.resource_allocator
    }
    pub fn gfx(&self) -> &dyn GfxInterface {
        self.check_validity();
        unsafe { self.gfx.assume_init_ref().as_ref() }
    }
    pub fn platform(&self) -> &dyn Platform {
        self.check_validity();
        unsafe { self.platform.assume_init_ref().as_ref() }
    }
    pub fn asset_manager(&self) -> &AssetManager {
        self.check_validity();
        unsafe { self.asset_manager.assume_init_ref() }
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
        // Wait render thread completion
        Gfx::get().stop_rendering_tasks();
    }

    pub fn delta_second(&self) -> f64 {
        self.game_delta.delta_seconds
    }
}

#[derive(Default)]
pub struct Camera {}

impl Drop for Engine {
    fn drop(&mut self) {
        self.app.stopped();

        // Unload worlds and views
        Gfx::get().shutdown();
        self.worlds.write().unwrap().clear();

        // Started de-initialization
        info!("Start deinitialization");
        *self.initialized_lock.lock().unwrap() = false;
        unsafe {
            self.asset_manager.assume_init_drop();
            self.gfx.assume_init_drop();
            self.platform.assume_init_drop();
        }

        // Now de-initialized
        self.pre_initialized.store(false, Ordering::SeqCst);
        unsafe { ENGINE_INSTANCE = std::ptr::null_mut() }
        info!("closed engine");
    }
}
