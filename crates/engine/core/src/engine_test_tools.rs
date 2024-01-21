use std::ptr::null_mut;
use std::sync::{Arc, Condvar, Mutex, RwLock, Weak};
use std::thread;
use crate::engine::{App, Builder, Engine, EngineRef};

#[derive(Default)]
pub struct TestEngineApp {}

static mut LOADED_ENGINE: RwLock<Option<EngineRef>> = RwLock::new(None);
static TEST_MUTEX: Mutex<i64> = Mutex::new(0);
static TEST_COND: Condvar = Condvar::new();

impl App for TestEngineApp {
    fn pre_initialize(&mut self, builder: &mut Builder) {}
    fn initialized(&mut self) {}
    fn new_frame(&mut self, delta_seconds: f64) {
        let mut l = TEST_MUTEX.lock().unwrap();
        while *l != 0 {
            l = TEST_COND.wait(l).unwrap();
        }
        Engine::get().shutdown();
    }
    fn request_shutdown(&self) {}
    fn stopped(&self) {}
}

pub fn run_with_test_engine<T: FnMut() + 'static>(callback: T) {
    *TEST_MUTEX.lock().unwrap() += 1;
    unsafe {
        let engine = &mut *LOADED_ENGINE.write().unwrap();
        if engine.is_none() {
            *engine = Some((Engine::new(TestEngineApp {})));

            thread::spawn(|| {
                let r = &*LOADED_ENGINE.read().unwrap();
                if let Some(e) = r {
                    (*(e as *const EngineRef as *mut EngineRef)).start();
                }
            });
        }
    }

    unsafe {
        let mut engine_ptr = null_mut();
        {
            let engine = &*LOADED_ENGINE.read().unwrap();
            if let Some(e) = engine { engine_ptr = e as *const EngineRef as *mut EngineRef; }
        }
        (*engine_ptr).wait_initialization();
    }
    let mut c = callback;
    c();
    *TEST_MUTEX.lock().unwrap() -= 1;
    TEST_COND.notify_all();
}