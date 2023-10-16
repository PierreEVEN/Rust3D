use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use shader_base::{BindPoint, DescriptorType, ShaderInterface};
use shader_base::pass_id::PassID;

use crate::command_buffer::GfxCommandBuffer;
use crate::Gfx;
use crate::image::GfxImage;
use crate::image_sampler::ImageSampler;
use crate::shader::ShaderProgram;
use crate::shader_instance::ShaderInstance;

#[derive(Clone)]
pub enum MaterialResourceData {
    Sampler(Arc<dyn ImageSampler>),
    SampledImage(Arc<dyn GfxImage>),
}

#[derive(Default)]
pub struct MaterialResources {
    resources: RwLock<HashMap<BindPoint, (DescriptorType, u32, Option<MaterialResourceData>, HashSet<PassID>)>>,
}

impl Clone for MaterialResources {
    fn clone(&self) -> Self {
        Self {
            resources: RwLock::new((*self.resources.read().unwrap()).clone()),
        }
    }
}

impl MaterialResources {
    pub fn add_binding(&self, descriptor_type: DescriptorType, bind_point: BindPoint, location: u32, passes: HashSet<PassID>) {
        let resources = &mut *self.resources.write().unwrap();
        resources.insert(bind_point, (descriptor_type, location, None, passes));
    }
    pub fn clear(&self) {
        self.resources.write().unwrap().clear();
    }
    pub fn bind_resource(&self, bind_point: &BindPoint, resource: MaterialResourceData) {
        let resources = &mut *self.resources.write().unwrap();
        match resources.get_mut(bind_point) {
            None => { logger::warning!("This material have no bind point '{:?}' available", bind_point) }
            Some((_, _, current, _)) => {
                *current = Some(resource);
            }
        }
    }
    pub fn get_bindings_for_pass(&self, pass: &PassID) -> Vec<(u32, MaterialResourceData)> {
        let mut bindings = vec![];

        for (_, location, resource, passes) in self.resources.read().unwrap().values() {
            if passes.contains(pass) {
                if let Some(resource) = resource {
                    bindings.push((*location, resource.clone()))
                }
            }
        }
        bindings
    }
}

pub struct PassMaterialData {
    instance: Arc<dyn ShaderInstance>,
    master: Arc<dyn ShaderProgram>,
}

impl Clone for PassMaterialData {
    fn clone(&self) -> Self {
        Self {
            instance: self.master.instantiate(),
            master: self.master.clone(),
        }
    }
}

#[derive(Default)]
pub struct Material {
    passes: RwLock<HashMap<PassID, Option<PassMaterialData>>>,
    shader_interface: RwLock<Option<Arc<dyn ShaderInterface>>>,
    resources: Arc<MaterialResources>,
}

impl Clone for Material {
    fn clone(&self) -> Self {
        Self {
            passes: RwLock::new((&*self.passes.read().unwrap()).clone()),
            shader_interface: RwLock::new(self.shader_interface.read().unwrap().clone()),
            resources: Arc::new((*self.resources).clone()),
        }
    }
}

impl Material {
    pub fn set_shader<T: 'static + ShaderInterface>(&self, shader: T) {
        if !shader.get_errors().is_empty() {
            for error in shader.get_errors() {
                logger::error!("{:?}", error);
            }
            return;
        }        
        self.resources.clear();
        for (bp, (descriptor, location, passes)) in shader.get_bindings() {
            self.resources.add_binding(descriptor, bp, location, passes)
        }        
        *self.shader_interface.write().unwrap() = Some(Arc::new(shader));
    }

    pub fn bind_to(&self, command_buffer: &dyn GfxCommandBuffer) {
        let pass = command_buffer.get_pass_id();
        if let Some(program) = self.get_program(&pass) {
            command_buffer.bind_program(&program);
            if let Some(instance) = self.get_instance(&pass) {
                command_buffer.bind_shader_instance(&instance);
            }
        }
    }

    pub fn bind_texture(&self, bind_point: &BindPoint, texture: Arc<dyn GfxImage>) {
        self.resources.bind_resource(bind_point, MaterialResourceData::SampledImage(texture))
    }

    pub fn bind_sampler(&self, bind_point: &BindPoint, sampler: Arc<dyn ImageSampler>) {
        self.resources.bind_resource(bind_point, MaterialResourceData::Sampler(sampler))
    }

    pub fn get_program(&self, pass_id: &PassID) -> Option<Arc<dyn ShaderProgram>> {
        // If program was previously loaded
        if let Some(program) = self.passes.read().unwrap().get(pass_id) {
            return match program {
                None => { None }
                Some(mat_data) => { Some(mat_data.master.clone()) }
            };
        }

        // Else load it
        match self.shader_interface.read().unwrap().as_ref() {
            None => { None }
            Some(shi) => {
                return match Gfx::get().get_program_pool().find_or_create_program(pass_id, shi) {
                    None => {
                        self.passes.write().unwrap().insert(pass_id.clone(), None);
                        None
                    }
                    Some(program) => {
                        self.passes.write().unwrap().insert(pass_id.clone(), Some(PassMaterialData {
                            instance: program.instantiate(),
                            master: program.clone(),
                        }));
                        Some(program)
                    }
                };
            }
        }
    }

    pub fn get_instance(&self, pass_id: &PassID) -> Option<Arc<dyn ShaderInstance>> {
        if let Some(pass) = self.passes.read().unwrap().get(pass_id) {
            return match pass {
                None => { None }
                Some(data) => { Some(data.instance.clone()) }
            };
        }

        // Try refresh program
        match self.get_program(&pass_id) {
            None => { None }
            Some(_) => { self.get_instance(pass_id) } // Try again
        }
    }
}
