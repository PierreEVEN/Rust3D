use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use logger::fatal;
use crate::engine::Engine;

/**
Store a game resource, and handle it's asynchronous loading.
It is used to track all existing resources loaded in the engine.
*/

pub trait ResourceTypeName {
    fn type_name(&self) -> &str;
}

impl<T> ResourceTypeName for T where T: Resource {
    fn type_name(&self) -> &str {
        std::any::type_name::<T>()
    }
}

pub trait Resource : ResourceTypeName {
    fn name(&self) -> &str;
}

pub trait ResourceFactory<T: Resource> {
    fn instantiate(self) -> T;
}

pub struct ResourceStorage<T: Resource + 'static> {
    loading: AtomicBool,
    loaded_object: RwLock<*mut T>,
}

impl<T: Resource> Default for ResourceStorage<T> {
    fn default() -> Self {
        Self {
            loading: AtomicBool::new(false),
            loaded_object: RwLock::new(null_mut()),
        }
    }
}

impl<T: Resource + 'static> ResourceStorage<T> {
    
    // Load this resource from a factory. Can be executed on any thread
    pub fn load<F: ResourceFactory<T>>(&self, factory: F) {
        self.loading.store(true, Ordering::SeqCst);
        let new_resource = Box::into_raw(Box::new(factory.instantiate()));

        // Place the old resource in the flush pool and swap it with the new one
        let resource_replace_lock = &mut *self.loaded_object.write().unwrap();

        if !resource_replace_lock.is_null() {
            Engine::get().resource_allocator().drop_resource(*resource_replace_lock);
            *resource_replace_lock = null_mut();
        }

        Engine::get().resource_allocator().register(new_resource);
        *resource_replace_lock = new_resource;

        self.loading.store(false, Ordering::SeqCst);
    }

    // Will destroy the resource once the full game loop is finished.
    pub fn destroy(&self) {
        let resource_replace_lock = &mut *self.loaded_object.write().unwrap();
        if (*resource_replace_lock).is_null() {
            return;
        }
        Engine::get().resource_allocator().drop_resource(*resource_replace_lock);
        *resource_replace_lock = null_mut();
    }

    pub fn is_valid(&self) -> bool {
        return !self.loaded_object.read().unwrap().is_null();
    }

    pub fn is_loading(&self) -> bool {
        return self.loading.load(Ordering::SeqCst);
    }

    pub fn unwrap(&self) -> &'static T {
        let opt = *self.loaded_object.read().unwrap();
        if opt.is_null() {
            fatal!("Failed to unwrap resource : null()")
        }
        else {
            unsafe { &*opt }
        }
    }
}

impl<T: Resource + 'static> Drop for ResourceStorage<T> {
    fn drop(&mut self) {
        self.destroy();
    }
}