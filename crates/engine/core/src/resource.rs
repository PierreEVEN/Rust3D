use std::collections::HashSet;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use logger::fatal;
use crate::engine::Engine;

pub trait ResourceFactory<T: Resource> {
    fn instantiate(self) -> T;
}

trait TResourceDropHelper {}

struct ResourceDropHelper<T: Resource> {
    pub resource: *mut T,
}

impl<T: Resource> Drop for ResourceDropHelper<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.resource));
        }
    }
}

impl<T: Resource> TResourceDropHelper for ResourceDropHelper<T> {}

#[derive(Default)]
pub struct ResourceAllocator {
    pending_delete: RwLock<Vec<Box<dyn TResourceDropHelper>>>,
    allocated_resources: RwLock<HashSet<*mut dyn Resource>>,
}

impl ResourceAllocator {
    pub fn drop_resource<T: Resource + 'static>(&self, old_resource: *mut T) {
        self.allocated_resources.write().unwrap().remove(&(old_resource as *mut dyn Resource));
        self.pending_delete.write().unwrap().push(Box::new(ResourceDropHelper::<T> { resource: old_resource }));
    }

    pub fn flush(&self) {
        self.pending_delete.write().unwrap().clear();
    }

    pub fn register<T: Resource + 'static>(&self, resource: *mut T) {
        self.allocated_resources.write().unwrap().insert(resource);
    }
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
    pub fn load<F: ResourceFactory<T>>(&self, factory: F) {
        self.loading.store(true, Ordering::SeqCst);
        let mut new_resource = Box::into_raw(Box::new(factory.instantiate()));

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

impl<T: Resource + 'static> Drop for ResourceStorage<T> {
    fn drop(&mut self) {
        self.destroy();
    }
}

pub trait Resource {
    fn type_name(&self) -> &str;
    fn name(&self) -> &str;
}

#[cfg(test)]
mod resources_tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
    use std::sync::atomic::Ordering::SeqCst;
    use logger::{error, info, warning};
    use crate::engine::{App, Builder, Engine};
    use crate::resource::{Resource, ResourceFactory, ResourceStorage};

    struct TestResource {
        pub value: i64,
        allocation_status: Arc<AtomicI64>
    }

    impl Resource for TestResource {
        fn type_name(&self) -> &str {
            "Custom resource type"
        }

        fn name(&self) -> &str {
            "Custom resource name"
        }
    }

    impl Drop for TestResource {
        fn drop(&mut self) {
            self.allocation_status.fetch_sub(1, SeqCst);
        }
    }

    struct TestResourceFactory {
        allocation_status: Arc<AtomicI64>,
        value: i64,
    }

    impl ResourceFactory<TestResource> for TestResourceFactory {
        fn instantiate(self) -> TestResource {
            self.allocation_status.fetch_add(1, Ordering::SeqCst);
            TestResource { value: self.value, allocation_status: self.allocation_status.clone() }
        }
    }

    #[test]
    fn create() {
        crate::engine_test_tools::run_with_test_engine(|| {
            for i in 0..10000 {
                let allocation_status = Arc::new(AtomicI64::new(0));
                let storage = ResourceStorage::default();
                storage.load(TestResourceFactory { value: 12, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 12);
                assert_eq!(allocation_status.load(SeqCst), 1);
            }
        });
    }

    #[test]
    fn destroy() {
        crate::engine_test_tools::run_with_test_engine(|| {
            for i in 0..10000 {
                let allocation_status = Arc::new(AtomicI64::new(0));
                let storage = ResourceStorage::default();
                storage.load(TestResourceFactory { value: 12, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 12);
                storage.destroy();
                Engine::get().resource_allocator().flush();
                assert_eq!(allocation_status.load(SeqCst), 0);
            }
        });
    }

    #[test]
    fn replace() {
        crate::engine_test_tools::run_with_test_engine(|| {
            for i in 0..10000 {
                let allocation_status = Arc::new(AtomicI64::new(0));
                let storage = ResourceStorage::default();
                storage.load(TestResourceFactory { value: 12, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 12);
                storage.destroy();
                storage.load(TestResourceFactory { value: 50, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 50);
                storage.load(TestResourceFactory { value: 25, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 25);
                storage.destroy();
                Engine::get().resource_allocator().flush();
                assert_eq!(allocation_status.load(SeqCst), 0);
            }
        });
    }

    #[test]
    fn accumulate() {
        crate::engine_test_tools::run_with_test_engine(|| {
            for i in 0..100 {
                let allocation_status = Arc::new(AtomicI64::new(0));
                let mut storages = vec![];
                for j in 0..100 {
                    {
                        let storage = ResourceStorage::default();
                        storage.load(TestResourceFactory { value: j, allocation_status: allocation_status.clone() });
                        assert!(storage.is_valid());
                        storages.push(storage);
                        assert!(storages.last().unwrap().is_valid());
                    }
                    assert!(storages.last().unwrap().is_valid());
                }
                assert_eq!(allocation_status.load(SeqCst), 100);

                for j in 0..100 {
                    if !storages[j].is_valid() {
                        error!("storage[{j}] is not valid");
                    }
                    assert!(storages[j].is_valid());
                    assert_eq!(storages[j].unwrap().value, j as i64);
                    storages[j].destroy();
                }
                Engine::get().resource_allocator().flush();
                assert_eq!(allocation_status.load(SeqCst), 0);
            }
        });
    }
}