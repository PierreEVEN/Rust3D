use std::collections::HashSet;
use std::sync::RwLock;
use crate::resource::resource::Resource;

trait TResourceDropHelper {}

struct ResourceDropHelper<T: Resource> {
    pub resource: *mut T,
}

impl<T: Resource> TResourceDropHelper for ResourceDropHelper<T> {}

impl<T: Resource> Drop for ResourceDropHelper<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.resource));
        }
    }
}

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
