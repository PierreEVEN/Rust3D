use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use logger::fatal;
use crate::engine::Engine;
use crate::gfx::surface::Frame;
use crate::resource::resource::{Resource, ResourceFactory, ResourceStorage};







pub struct SwResourceStorage<T: Resource + 'static> {
    resources: Vec<(AtomicBool, RwLock<*mut T>)>,
}

impl<T: Resource> Default for SwResourceStorage<T> {
    fn default() -> Self {
        Self {
            loading: AtomicBool::new(false),
            loaded_object: RwLock::new(null_mut()),
        }
    }
}

impl<T: Resource + 'static> SwResourceStorage<T> {
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




















struct TestStr {}

impl Resource for TestStr {
    fn name(&self) -> &str {
        todo!()
    }
}

struct TestStrFactory {}

impl ResourceFactory<TestStr> for TestStrFactory {
    fn instantiate(self) -> TestStr {
        TestStr {}
    }
}


fn frame_function(frame: Frame) {
    
    let test = ResourceStorage::swapchain(frame);

    {
        test.load(TestStrFactory {});
        let data = *test;

        test.is_loading_for(frame);
        test.unwrap_for(frame);
    }
}