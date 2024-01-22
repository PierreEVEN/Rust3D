use std::collections::HashMap;
use std::default::Default;
use std::ops::Deref;
use std::sync::RwLock;
use crate::gfx::surface::Frame;


pub trait GfxImageBuilder<T: Clone> {
    fn build(&self, swapchain_ref: &Frame) -> T;
}

struct DefaultSwapchainResourceBuilder {}

impl<T: Clone> GfxImageBuilder<T> for DefaultSwapchainResourceBuilder {
    fn build(&self, _swapchain_ref: &Frame) -> T {
        todo!()
    }
}

pub struct GfxResource<T> {
    resources: RwLock<HashMap<Frame, T>>,
    builder: RwLock<Box<dyn GfxImageBuilder<T>>>,
    static_resource: bool,
}

impl<T: Clone> Default for GfxResource<T> {
    fn default() -> Self {
        Self {
            resources: RwLock::default(),
            builder: RwLock::new(Box::new(DefaultSwapchainResourceBuilder {})),
            static_resource: true,
        }
    }
}

impl<T: Clone> GfxResource<T> {
    pub fn new<U: GfxImageBuilder<T> + 'static>(builder: U) -> Self {
        let builder = Box::new(builder) as Box<dyn GfxImageBuilder<T>>;

        Self {
            static_resource: false,
            builder: RwLock::new(builder),
            resources: RwLock::default(),
        }
    }
    pub fn new_static<U: GfxImageBuilder<T> + 'static>(builder: U) -> Self {
        let static_ref = Frame::new(0, 0);
        Self {
            static_resource: true,
            resources: RwLock::new(HashMap::from([(
                static_ref.clone(),
                builder.build(&static_ref),
            )])),
            builder: RwLock::new(Box::new(builder)),
        }
    }

    pub fn get_static(&self) -> T {
        if !self.static_resource {
            logger::fatal!("The current resource is not static. You should call get(...) instead.");
        }
        self.resources
            .read()
            .unwrap()
            .deref()
            .get(&Frame::null())
            .unwrap()
            .clone()
    }
    pub fn get(&self, reference: &Frame) -> T {
        if self.static_resource {
            logger::fatal!(
                "The current resource is static. You should call get_static(...) instead."
            );
        }

        {
            match self.resources.read().unwrap().deref().get(reference) {
                None => {}
                Some(resource) => {
                    return resource.clone();
                }
            }
        }

        self.resources.write().unwrap().insert(
            reference.clone(),
            self.builder.read().unwrap().as_ref().build(reference),
        );
        self.resources
            .read()
            .unwrap()
            .get(reference)
            .unwrap()
            .clone()
    }

    pub fn invalidate<U: GfxImageBuilder<T> + 'static>(&self, builder: U) {
        let mut resource_map = self.resources.write().unwrap();
        (*resource_map).clear();
        let mut builder_ref = self.builder.write().unwrap();
        *builder_ref = Box::new(builder);

        if self.static_resource {
            let static_ref = Frame::null();
            resource_map.insert(static_ref.clone(), builder_ref.build(&static_ref));
        }
    }
    
    pub fn is_static(&self) -> bool {
        self.static_resource
    }
}
