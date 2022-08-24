use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{RwLock};

use gfx::GfxRef;
use gfx::surface::{GfxImageID};

pub trait GfxImageBuilder<T: Clone> {
    fn build(&self, gfx: &GfxRef, swapchain_ref: &GfxImageID) -> T;
}

struct DefaultSwapchainResourceBuilder {}

impl<T: Clone> GfxImageBuilder<T> for DefaultSwapchainResourceBuilder {
    fn build(&self, _gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> T {
        todo!()
    }
}

pub struct VkSwapchainResource<T> {
    resources: RwLock<HashMap<GfxImageID, T>>,
    builder: Box<dyn GfxImageBuilder<T>>,
    static_resource: bool,
}

impl<T: Clone> Default for VkSwapchainResource<T> {
    fn default() -> Self {
        Self {
            resources: RwLock::default(),
            builder: Box::new(DefaultSwapchainResourceBuilder {}),
            static_resource: true,
        }
    }
}

impl<T: Clone> VkSwapchainResource<T> {
    pub fn  new(builder: Box<dyn GfxImageBuilder<T>>) -> Self {
        Self {
            static_resource: false,
            builder,
            resources: RwLock::default(),
        }
    }
    pub fn new_static(gfx: &GfxRef, builder: Box<dyn GfxImageBuilder<T>>) -> Self {
        let static_ref = GfxImageID::new(gfx.clone(), 0, 0);
        Self {
            static_resource: true,
            resources: RwLock::new(HashMap::from([(static_ref.clone(), builder.build(gfx, &static_ref))])),
            builder,
        }
    }

    pub fn get(&self, reference: &GfxImageID) -> T {
        if self.static_resource {
            let image_id = GfxImageID::new(reference.gfx().clone(), 0, 0);
            return self.resources.read().unwrap().deref().get(&image_id).unwrap().clone();
        }

        {
            match self.resources.read().unwrap().deref().get(&reference) {
                None => {}
                Some(resource) => { return resource.clone(); }
            }
        }

        self.resources.write().unwrap().insert(reference.clone(), self.builder.as_ref().build(&reference.gfx(), reference));
        self.resources.read().unwrap().get(reference).unwrap().clone()
    }
}