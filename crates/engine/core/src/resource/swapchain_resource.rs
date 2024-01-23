use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use logger::fatal;
use crate::engine::Engine;
use crate::gfx::Gfx;
use crate::gfx::surface::Frame;
use crate::resource::resource::{Resource, ResourceFactory};

/*

struct SubSwResourceStorage <T : Resource + 'static> {
    loading_call_count: i64,
    loading_state: Option<Box<dyn Resource>>, // Pending load count, current resource
    resource: *mut T
}

impl<T: Resource + 'static> SubSwResourceStorage<T> {
    pub fn load_for_factory(&mut self, factory: Arc<dyn ResourceFactory<T>>) {
        self.loading_call_count += 1;
        self.loading_state = Some(Box::new(factory.instantiate()));
        self.loading_call_count -= 1;



    }
}


pub struct SwResourceStorage<T: Resource + 'static> {
    resources: Arc<Vec<(RwLock<(bool, *mut T)>, RwLock<Option<Box<dyn Resource>>>)>>,
    is_per_image: bool,
    reconstruction_factory: RwLock<Option<Arc<dyn ResourceFactory<T>>>>,
}

impl<T: Resource + 'static> SwResourceStorage<T> {
    pub fn swapchain() -> Self {
        Self { resources: Arc::new(vec![]), is_per_image: false, reconstruction_factory: None }
    }
}

impl<T: Resource + 'static> SwResourceStorage<T> {
    pub fn invalidate(&self) {
        let factory = &mut *self.reconstruction_factory.read().unwrap();
        match factory {
            None => {
                self.destroy();
            }
            Some(factory) => {

                // Invalidate existing resources
                for sub_resource in self.resources {
                    let (valid, _resource) = &mut *sub_resource.write().unwrap();
                    *valid = false;
                }

                // Prepare the newly spawned instances
                for instance in &*self.next_instances {
                    let new_instance = factory.instantiate();
                    let next_instances = &mut *instance.write().unwrap();
                    *next_instances = Some(new_instance);
                }

                // send command to update instances next frame

                let factory = factory.clone();
                Gfx::get().on_next_frame_reset(move |_| {
                    let instance = factory.instantiate();
                })
            }
        }
    }
}

pub fn load<F: ResourceFactory<T>>(&self, factory: F) {
    *self.reconstruction_factory.write().unwrap() = Some((Box::new(factory), Frame::new(0, 0)));


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
    } else {
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
    let test = SwResourceStorage::swapchain(frame);

    {
        test.load(TestStrFactory {});
        let data = *test;

        test.is_loading_for(frame);
        test.unwrap_for(frame);
    }
}

 */