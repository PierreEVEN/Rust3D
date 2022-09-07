use std::collections::HashMap;
use std::ops::Deref;
use std::sync::RwLock;

use crate::GfxRef;
use crate::surface::GfxImageID;

pub trait GfxImageBuilder<T: Clone> {
    fn build(&self, gfx: &GfxRef, swapchain_ref: &GfxImageID) -> T;
}

struct DefaultSwapchainResourceBuilder {}

impl<T: Clone> GfxImageBuilder<T> for DefaultSwapchainResourceBuilder {
    fn build(&self, _gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> T {
        todo!()
    }
}

pub struct GfxResource<T> {
    resources: RwLock<HashMap<GfxImageID, T>>,
    builder: RwLock<Box<dyn GfxImageBuilder<T>>>,
    static_resource: bool,
    gfx: Option<GfxRef>,
}

impl<T: Clone> Default for GfxResource<T> {
    fn default() -> Self {
        Self {
            resources: RwLock::default(),
            builder: RwLock::new(Box::new(DefaultSwapchainResourceBuilder {})),
            static_resource: true,
            gfx: None,
        }
    }
}

impl<T: Clone> GfxResource<T> {
    pub fn new<U: GfxImageBuilder<T> + 'static>(gfx: &GfxRef, builder: U) -> Self {
        let builder = Box::new(builder) as Box<dyn GfxImageBuilder<T>>;

        Self {
            static_resource: false,
            builder: RwLock::new(builder),
            resources: RwLock::default(),
            gfx: Some(gfx.clone()),
        }
    }
    pub fn new_static<U: GfxImageBuilder<T> + 'static>(gfx: &GfxRef, builder: U) -> Self {
        let static_ref = GfxImageID::new(0, 0);
        Self {
            static_resource: true,
            resources: RwLock::new(HashMap::from([(static_ref.clone(), builder.build(gfx, &static_ref))])),
            builder: RwLock::new(Box::new(builder)),
            gfx: Some(gfx.clone()),
        }
    }

    pub fn get_static(&self) -> T {
        if !self.static_resource {
            panic!("The current resource is not static. You should call get(...) instead.");
        }
        self.resources.read().unwrap().deref().get(&GfxImageID::null()).unwrap().clone()
    }
    pub fn get(&self, reference: &GfxImageID) -> T {
        if self.static_resource {
            panic!("The current resource is static. You should call get_static(...) instead.");
        }

        {
            match self.resources.read().unwrap().deref().get(&reference) {
                None => {}
                Some(resource) => { return resource.clone(); }
            }
        }

        self.resources.write().unwrap().insert(reference.clone(), self.builder.read().unwrap().as_ref().build(match &self.gfx {
            None => { panic!("gfx is not valid") }
            Some(gfx) => { gfx }
        }, reference));
        self.resources.read().unwrap().get(reference).unwrap().clone()
    }

    pub fn invalidate<U: GfxImageBuilder<T> + 'static>(&self, gfx: &GfxRef, builder: U) {
        let mut resource_map = self.resources.write().unwrap();
        (*resource_map).clear();
        let mut builder_ref = self.builder.write().unwrap();
        *builder_ref = Box::new(builder);

        if self.static_resource {
            let static_ref = GfxImageID::null();
            resource_map.insert(static_ref.clone(), builder_ref.build(gfx, &static_ref));
        }
    }

    pub fn is_static(&self) -> bool {
        self.static_resource
    }
}